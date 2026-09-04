//! Keeps the provider's prompt cache warm while the session sits idle.
//!
//! Chat-wire providers key their prompt cache off the literal message prefix; there is no
//! `prompt_cache_key` on that wire to hand the server a name for the entry. The entry therefore
//! lives on a short server-side TTL, and any pause long enough to outlast it — a user reading a
//! diff, writing the next message, or answering a prompt — makes the following request re-bill the
//! entire prefix at the uncached rate. On a normal working session that is by far the largest
//! single source of cache-miss cost.
//!
//! The fix is to replay the last request's exact wire body with the output capped at one token.
//! Replaying it is the only thing that can refresh the right entry, since the body *is* the cache
//! identity. Response headers arrive after prefill, which is the moment the entry is refreshed, so
//! the stream is dropped unread.
//!
//! This lives below the session on purpose. It is handed a transport, a provider, an auth provider
//! and a body, and it holds no reference to conversation history, the rollout, the event channel,
//! or token accounting. A keep-alive is invisible to the rest of Suffice by construction rather
//! than by discipline: there is nothing here it could reach even by mistake.
//!
//! The budget below prices the risk that an idle user never comes back. It is suspended by
//! [`PromptCacheKeepAlive::hold`] while the harness is waiting on work it started itself, where
//! there is no such risk.

use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::OnceLock;
use std::sync::PoisonError;
use std::sync::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use codex_api::Compression;
use codex_api::Provider;
use codex_api::ReqwestTransport;
use codex_api::ResponsesClient;
use codex_api::SharedAuthProvider;
use http::HeaderMap;
use serde_json::Value;

/// How long the session may sit idle before the cache entry is refreshed.
///
/// Set just inside the prompt-cache TTL measured on Z.ai rather than to minimize requests; a
/// keep-alive that lands after the entry expired has paid for nothing. Across 976 consecutive
/// same-turn requests on the Chat wire, every idle gap up to 448 seconds still hit the cache in
/// full and the first miss was at 1,023 seconds, so seven minutes sits under the shortest gap that
/// has ever been seen to lose the entry.
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(420);

/// How many refreshes may fire before a real request has to re-arm the session.
///
/// A refresh resends the prefix while the entry is still warm, so it is billed at the cached rate
/// and costs `prefix * c`. The miss it prevents costs `prefix * (1 - c)`. Z.ai bills cached input
/// at a fifth of uncached, so the two break even at `p * (1 - c) / c` refreshes, where `p` is the
/// chance the user returns at all: four when returning is certain.
///
/// Four is deliberately the `p = 1` bound rather than a hedge against abandoned sessions. Below it
/// the feature is leaving money on the table for every user who does come back, and an abandoned
/// session stops costing anything after twenty-eight minutes either way.
const MAX_CONSECUTIVE_KEEP_ALIVES: u32 = 4;

/// Output cap for a replay.
///
/// The point of the request is prefill, not generation. If a model rejects a cap this small for
/// being below its thinking budget the request fails before prefill and warms nothing, in which
/// case raise this — it is a floor on what the provider accepts, not a tuning knob.
const KEEP_ALIVE_MAX_TOKENS: u32 = 1;

/// Everything needed to replay the last real request, captured at its call site.
#[derive(Clone)]
struct Armed {
    /// Bumped by every real request. The idle task compares it across its sleep, so a turn that
    /// starts mid-wait re-anchors the schedule and restores the full budget without any
    /// cancellation plumbing.
    generation: u64,
    armed_at: Instant,
    transport: ReqwestTransport,
    provider: Provider,
    auth: SharedAuthProvider,
    /// Filled by the endpoint once the body is final, so this is the bytes that actually went on
    /// the wire rather than a second, possibly divergent, translation of the same request.
    body: Arc<OnceLock<Value>>,
}

pub(crate) struct PromptCacheKeepAlive {
    /// The generation lives inside the mutex so a single lock yields a consistent snapshot and
    /// there is no ordering between separate atomics to reason about.
    armed: StdMutex<Option<Armed>>,
    task_started: AtomicBool,
    /// How many pieces of work the harness is currently waiting on. Read only to decide whether
    /// the budget applies, so it needs no ordering relative to the snapshot above.
    holds: AtomicUsize,
}

/// Suspends the refresh budget for as long as it is held.
///
/// The budget prices the chance that a paused session is abandoned rather than idle: past some
/// point the refreshes are warming an entry nobody will ever claim. That reasoning does not apply
/// while the harness is blocked on something it started itself, such as a long shell command. There
/// the next request is not a guess but a certainty, and so is the cache miss the refresh prevents,
/// so the budget is suspended rather than merely widened — a command that runs for an hour is
/// covered for an hour.
///
/// The suspension is a guard rather than a pair of calls because tool dispatch can fail or be
/// cancelled, and a hold leaked by an early return would disable the budget for the rest of the
/// session.
pub(crate) struct KeepAliveHold(Arc<PromptCacheKeepAlive>);

impl Drop for KeepAliveHold {
    fn drop(&mut self) {
        self.0.holds.fetch_sub(1, Ordering::SeqCst);
    }
}

impl std::fmt::Debug for PromptCacheKeepAlive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptCacheKeepAlive")
            .finish_non_exhaustive()
    }
}

/// What the idle task should do when it wakes.
enum Tick {
    /// A real request landed during the sleep; re-anchor on it and start the budget over.
    Rearm,
    /// Nothing to refresh, or the budget is spent. Keep waiting for a real request.
    Park,
    Fire(Armed),
}

impl PromptCacheKeepAlive {
    pub(crate) fn new() -> Self {
        Self {
            armed: StdMutex::new(None),
            task_started: AtomicBool::new(false),
            holds: AtomicUsize::new(0),
        }
    }

    /// Marks the harness as waiting on work it started, suspending the budget until the guard drops.
    pub(crate) fn hold(self: &Arc<Self>) -> KeepAliveHold {
        self.holds.fetch_add(1, Ordering::SeqCst);
        KeepAliveHold(Arc::clone(self))
    }

    /// Records a real request as the one to replay while the session is idle.
    ///
    /// The idle task is spawned here rather than at construction because clients are built from
    /// synchronous code with no Tokio runtime available.
    pub(crate) fn arm(
        self: &Arc<Self>,
        transport: ReqwestTransport,
        provider: Provider,
        auth: SharedAuthProvider,
        body: Arc<OnceLock<Value>>,
    ) {
        {
            let mut armed = self.armed.lock().unwrap_or_else(PoisonError::into_inner);
            let generation = armed.as_ref().map_or(0, |armed| armed.generation) + 1;
            *armed = Some(Armed {
                generation,
                armed_at: Instant::now(),
                transport,
                provider,
                auth,
                body,
            });
        }

        if !self.task_started.swap(true, Ordering::SeqCst) {
            // Only a weak handle: dropping the session is what stops the task, so it must not be
            // what keeps it alive.
            tokio::spawn(run(Arc::downgrade(self)));
        }
    }

    /// Decides what to do now, advancing the caller's view of the schedule.
    ///
    /// All of the scheduling lives here so it can be exercised without a clock or a network.
    fn on_tick(&self, seen: &mut u64, fires: &mut u32) -> Tick {
        let armed = self.armed.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(armed) = armed.as_ref() else {
            return Tick::Park;
        };
        if armed.generation != *seen {
            *seen = armed.generation;
            *fires = 0;
            return Tick::Rearm;
        }
        if *fires >= MAX_CONSECUTIVE_KEEP_ALIVES && self.holds.load(Ordering::SeqCst) == 0 {
            return Tick::Park;
        }
        // Counted even when the budget is suspended: `next_deadline` reads this to place the next
        // wake-up, so a fire that did not advance it would leave the deadline in the past and spin.
        *fires += 1;
        Tick::Fire(armed.clone())
    }

    /// Anchors the schedule on the last real request so refreshes land at fixed offsets from it.
    ///
    /// A free-running ticker would put the first refresh anywhere in one to two intervals after
    /// the request, which can overshoot the cache TTL and defeat the whole feature.
    fn next_deadline(&self, fires: u32) -> Instant {
        let armed = self.armed.lock().unwrap_or_else(PoisonError::into_inner);
        match armed.as_ref() {
            Some(armed) => armed.armed_at + KEEP_ALIVE_INTERVAL * (fires + 1),
            None => Instant::now() + KEEP_ALIVE_INTERVAL,
        }
    }
}

async fn run(keep_alive: Weak<PromptCacheKeepAlive>) {
    let mut seen = 0u64;
    let mut fires = 0u32;
    loop {
        let Some(deadline) = keep_alive
            .upgrade()
            .map(|keep_alive| keep_alive.next_deadline(fires))
        else {
            return;
        };
        tokio::time::sleep_until(deadline.into()).await;

        let Some(keep_alive) = keep_alive.upgrade() else {
            return;
        };
        match keep_alive.on_tick(&mut seen, &mut fires) {
            Tick::Rearm | Tick::Park => {}
            Tick::Fire(armed) => {
                // The lock is released before this point: nothing here may be awaited under it.
                fire(armed).await;
            }
        }
    }
}

async fn fire(armed: Armed) {
    let Some(body) = armed.body.get() else {
        return;
    };
    let body = keep_alive_body(body);

    let mut provider = armed.provider;
    // A refresh that needs a second attempt has already lost the race with the cache TTL, and the
    // outer auth-recovery loop is bypassed simply by not being in it.
    provider.retry.max_attempts = 1;

    // Built without telemetry and without session headers: a keep-alive must not produce
    // telemetry rows, protocol events, or a request id that correlates with the real session.
    let client = ResponsesClient::new(armed.transport, provider, armed.auth);
    match client
        .stream(
            body,
            HeaderMap::new(),
            Compression::None,
            /*turn_state*/ None,
        )
        .await
    {
        // Headers arrive after prefill, so by here the entry is refreshed and the body is waste.
        Ok(stream) => drop(stream),
        Err(err) => tracing::debug!("prompt cache keep-alive request failed: {err}"),
    }
}

/// Caps the replay's output without touching anything the cache is keyed on.
///
/// `messages`, `tools`, and `thinking` must stay byte-identical or the replay warms a different
/// entry than the one the next real request will look for. `stream` in particular stays on: the
/// response is parsed as SSE either way.
fn keep_alive_body(body: &Value) -> Value {
    let mut body = body.clone();
    if let Some(body) = body.as_object_mut() {
        body.insert("max_tokens".to_string(), KEEP_ALIVE_MAX_TOKENS.into());
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoAuth;

    impl codex_api::AuthProvider for NoAuth {
        fn add_auth_headers(&self, _headers: &mut HeaderMap) {}
    }

    fn keep_alive() -> Arc<PromptCacheKeepAlive> {
        Arc::new(PromptCacheKeepAlive::new())
    }

    /// Arms with material that is never sent; only the schedule is under test here.
    fn arm(keep_alive: &Arc<PromptCacheKeepAlive>) {
        keep_alive.arm(
            ReqwestTransport::from_http_client(codex_login::default_client::create_client()),
            Provider {
                name: "test".to_string(),
                base_url: "https://example.invalid".to_string(),
                query_params: None,
                headers: HeaderMap::new(),
                retry: codex_api::RetryConfig {
                    max_attempts: 3,
                    base_delay: Duration::from_millis(1),
                    retry_429: true,
                    retry_5xx: true,
                    retry_transport: true,
                },
                stream_idle_timeout: Duration::from_secs(1),
                wire: codex_api::WireApi::Chat,
            },
            Arc::new(NoAuth),
            Arc::new(OnceLock::new()),
        );
    }

    #[test]
    fn keep_alive_body_caps_output_and_leaves_the_prefix_byte_identical() {
        let body = serde_json::json!({
            "model": "glm-5.3-flash",
            "messages": [{"role": "user", "content": "hi"}],
            "stream": true,
            "stream_options": {"include_usage": true},
            "thinking": {"type": "enabled"},
        });

        let mut capped = keep_alive_body(&body);

        assert_eq!(capped["max_tokens"], serde_json::json!(1));
        capped.as_object_mut().expect("object").remove("max_tokens");
        assert_eq!(capped, body);
    }

    #[tokio::test]
    async fn a_real_request_resets_the_keep_alive_budget() {
        let keep_alive = keep_alive();
        let (mut seen, mut fires) = (0, 0);

        arm(&keep_alive);
        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Rearm
        ));
        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Fire(_)
        ));
        assert_eq!(fires, 1);

        arm(&keep_alive);
        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Rearm
        ));
        assert_eq!(fires, 0);
    }

    #[tokio::test]
    async fn the_keep_alive_stops_once_the_idle_budget_is_spent() {
        let keep_alive = keep_alive();
        let (mut seen, mut fires) = (0, 0);

        arm(&keep_alive);
        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Rearm
        ));
        for _ in 0..MAX_CONSECUTIVE_KEEP_ALIVES {
            assert!(matches!(
                keep_alive.on_tick(&mut seen, &mut fires),
                Tick::Fire(_)
            ));
        }

        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Park
        ));
    }

    #[tokio::test]
    async fn waiting_on_a_tool_call_outlasts_the_idle_budget() {
        let keep_alive = keep_alive();
        let (mut seen, mut fires) = (0, 0);

        arm(&keep_alive);
        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Rearm
        ));

        let hold = keep_alive.hold();
        // A build that runs far past the idle budget still has a request waiting behind it, so
        // parking here would hand back the whole prefix moments before it is needed.
        for _ in 0..MAX_CONSECUTIVE_KEEP_ALIVES * 3 {
            assert!(matches!(
                keep_alive.on_tick(&mut seen, &mut fires),
                Tick::Fire(_)
            ));
        }
        // The schedule still advanced, or the idle task would wake on a deadline in the past and
        // spin through requests as fast as the provider answered them.
        assert_eq!(fires, MAX_CONSECUTIVE_KEEP_ALIVES * 3);

        drop(hold);
        assert!(matches!(
            keep_alive.on_tick(&mut seen, &mut fires),
            Tick::Park
        ));
    }
}
