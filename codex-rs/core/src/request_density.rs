//! How many requests a context window actually buys, averaged over the user's recent windows.
//!
//! Reasoning retention trades a one-off re-prefill against carrying tokens in the cached prefix
//! for the rest of the window, so what it needs is a count of *requests*, not a count of tokens:
//! a kept token is re-billed once per request that follows it. Converting the remaining token
//! budget into a request count needs the density of one against the other, and that varies by
//! more than an order of magnitude between a conversational session and a tool-heavy one.
//!
//! A compiled guess therefore biases every decision the same way for a whole class of user, so
//! the density is measured instead. Each time a window's worth of context has been consumed, the
//! number of requests it took is appended to a short history in the Codex home, and the mean of
//! the last [`MAX_SAMPLES`] windows is what the next session prices against.

use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::sync::Mutex;

use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;

/// Context consumed by one sample, which is the auto-compaction budget this fork ships. The unit
/// only has to be stable across releases for old samples to stay comparable to new ones.
pub(crate) const WINDOW_TOKENS: i64 = 80_000;
/// Windows the moving average covers.
const MAX_SAMPLES: usize = 20;
/// Density assumed before any window has been measured. Real sessions ran at 345-850 tokens of
/// context growth per request, so a full window buys roughly this many.
const DEFAULT_REQUESTS_PER_WINDOW: f64 = 160.0;
const FILENAME: &str = "request_density.json";

#[derive(Default, Serialize, Deserialize)]
struct Samples {
    #[serde(default)]
    requests_per_window: Vec<u32>,
}

#[derive(Default)]
struct Window {
    tokens: i64,
    requests: u32,
}

#[derive(Default)]
pub(crate) struct RequestDensity {
    /// Absent for sessions with no home to persist to, which then only ever see the default.
    path: Option<AbsolutePathBuf>,
    /// Read once at session start and held fixed, so a sample written mid-session cannot move a
    /// horizon that earlier decisions in the same prefix were already frozen against.
    average: Option<f64>,
    window: Mutex<Window>,
}

impl RequestDensity {
    pub(crate) async fn load(codex_home: &AbsolutePathBuf) -> Self {
        let path = codex_home.join(FILENAME);
        let samples = tokio::fs::read_to_string(&path)
            .await
            .ok()
            .and_then(|contents| serde_json::from_str::<Samples>(&contents).ok())
            .unwrap_or_default()
            .requests_per_window;
        Self {
            path: Some(path),
            average: mean(&samples),
            window: Mutex::default(),
        }
    }

    pub(crate) fn requests_per_window(&self) -> f64 {
        self.average.unwrap_or(DEFAULT_REQUESTS_PER_WINDOW)
    }

    /// Folds one request's context growth into the window being measured, writing a sample once a
    /// full window has been consumed.
    pub(crate) async fn observe(&self, growth_tokens: i64) {
        let Some(path) = self.path.clone() else {
            return;
        };
        let Some(sample) = self.fold(growth_tokens) else {
            return;
        };
        if let Err(error) = append(path, sample).await {
            tracing::debug!("failed to record request density sample: {error:#}");
        }
    }

    fn fold(&self, growth_tokens: i64) -> Option<u32> {
        let mut window = self.window.lock().ok()?;
        window.tokens = window.tokens.saturating_add(growth_tokens.max(0));
        window.requests = window.requests.saturating_add(1);
        if window.tokens < WINDOW_TOKENS {
            return None;
        }
        // The window is closed on the request that crossed the line, so it holds slightly more
        // than a window's tokens. Scale the count back to exactly one window.
        let sample = f64::from(window.requests) * WINDOW_TOKENS as f64 / window.tokens as f64;
        *window = Window::default();
        Some(sample.round().max(1.0) as u32)
    }
}

fn mean(samples: &[u32]) -> Option<f64> {
    let total: f64 = samples.iter().copied().map(f64::from).sum();
    (!samples.is_empty()).then(|| total / samples.len() as f64)
}

async fn append(path: AbsolutePathBuf, sample: u32) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::task::spawn_blocking(move || {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        file.lock()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut samples: Samples = serde_json::from_str(&contents).unwrap_or_default();
        samples.requests_per_window.push(sample);
        let excess = samples
            .requests_per_window
            .len()
            .saturating_sub(MAX_SAMPLES);
        samples.requests_per_window.drain(..excess);

        let encoded = serde_json::to_string(&samples)?;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(encoded.as_bytes())?;
        file.flush()
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_test_support::PathExt;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    #[tokio::test]
    async fn an_unmeasured_home_prices_against_the_default() {
        let home = TempDir::new().expect("create temp dir");

        let density = RequestDensity::load(&home.path().abs()).await;

        assert_eq!(density.requests_per_window(), DEFAULT_REQUESTS_PER_WINDOW);
    }

    #[tokio::test]
    async fn a_window_is_sampled_once_its_tokens_are_spent_and_read_back_next_session() {
        let home = TempDir::new().expect("create temp dir");
        let density = RequestDensity::load(&home.path().abs()).await;

        // Forty requests of 2K each spend exactly one window.
        for _ in 0..40 {
            density.observe(2_000).await;
        }

        let next_session = RequestDensity::load(&home.path().abs()).await;
        assert_eq!(next_session.requests_per_window(), 40.0);
    }

    #[tokio::test]
    async fn the_average_covers_only_the_most_recent_windows() {
        let home = TempDir::new().expect("create temp dir");
        let path = home.path().abs().join(FILENAME);
        std::fs::write(
            &path,
            serde_json::to_string(&Samples {
                requests_per_window: vec![1_000; MAX_SAMPLES],
            })
            .expect("encode samples"),
        )
        .expect("write samples");

        append(path, 60).await.expect("append sample");

        let density = RequestDensity::load(&home.path().abs()).await;
        let expected = ((MAX_SAMPLES - 1) as f64 * 1_000.0 + 60.0) / MAX_SAMPLES as f64;
        assert_eq!(density.requests_per_window(), expected);
    }
}
