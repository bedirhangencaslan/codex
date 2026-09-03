//! Decides whether reasoning is cheaper to keep than to drop, once per turn boundary.
//!
//! Reasoning used to be dropped the instant its turn ended, on the assumption that a smaller
//! context is always a cheaper context. That is only half of the price. On the Chat wire
//! reasoning is not a free-floating item: it is folded into the assistant message or tool call
//! it explains, so removing it *rewrites* that message, and prefix caching invalidates from the
//! first differing token to the end of the prompt. The first reasoning item of a turn sits at
//! the start of that turn, so dropping it re-bills the entire turn at the uncached rate.
//!
//! Measured against a real session, a `low`-effort turn produced reasoning with a median of 32
//! tokens. Keeping 32 tokens in a cached prefix costs a rounding error; dropping them cost a
//! five-figure re-prefill. At `max` effort the same reasoning is orders of magnitude larger and
//! the trade flips. So this is arithmetic, not a constant, and it is done here.
//!
//! Two properties make the pass cheap and safe:
//!
//! - **Cascade.** Invalidation runs from the first difference to the end, so once anything is
//!   dropped every droppable item after it costs no additional cache. There is therefore only
//!   ever one real decision to make: where to break.
//! - **Freezing.** A verdict is written onto the item and never recomputed. A decision that
//!   flipped between requests would produce exactly the cache miss the whole pass exists to
//!   avoid.

use codex_history::ResponseItemEnvelope;
use codex_protocol::models::ReasoningItemContent;
use codex_protocol::models::ReasoningItemReasoningSummary;
use codex_protocol::models::ResponseItem;
use codex_utils_output_truncation::approx_token_count;

use crate::context_manager::estimate_item_token_count;
use crate::request_density::WINDOW_TOKENS;

/// Cached input price as a fraction of uncached, for the providers this fork ships.
const CACHED_INPUT_PRICE_RATIO: f64 = 0.1;
/// Bounds on the horizon, so a tiny or absent density estimate cannot make keeping free.
const MIN_HORIZON: i64 = 1;
const MAX_HORIZON: i64 = 200;

pub(crate) struct RetentionInputs {
    /// Tokens left before the next auto-compaction.
    pub budget_remaining: i64,
    /// Requests this user gets out of a full context window, from
    /// [`crate::request_density::RequestDensity`].
    pub requests_per_window: f64,
}

/// Requests expected before compaction resets the prefix anyway.
///
/// Beyond that point the prefix is rewritten wholesale, so reasoning kept past it is never
/// re-billed and must not be counted as if it were. What is left of the budget is a fraction of
/// a window; the user's measured density turns that fraction into the count of requests that a
/// retained token would actually be re-billed on.
fn horizon(inputs: &RetentionInputs) -> i64 {
    let windows_remaining = inputs.budget_remaining.max(0) as f64 / WINDOW_TOKENS as f64;
    ((windows_remaining * inputs.requests_per_window).round() as i64)
        .clamp(MIN_HORIZON, MAX_HORIZON)
}

/// Cost of breaking the prefix at one candidate, in uncached-input token equivalents.
///
/// `suffix_tokens` is re-prefilled once, at the premium an uncached token carries over a cached
/// one. `retained_reasoning_tokens` instead rides along in the cached prefix for the whole
/// horizon. The break that minimizes this sum is the one to take.
fn break_cost(suffix_tokens: i64, retained_reasoning_tokens: i64, horizon: i64) -> f64 {
    let cached_ratio = CACHED_INPUT_PRICE_RATIO;
    (suffix_tokens as f64) * (1.0 - cached_ratio)
        + (horizon as f64) * cached_ratio * (retained_reasoning_tokens as f64)
}

/// Freezes a keep/drop verdict on every reasoning item whose turn has ended.
///
/// The candidates are evaluated from the newest backwards, because the cost of dropping an
/// early reasoning item is not its own size: dropping it also drops every reasoning item after
/// it for free, and it is only worth paying for if the *later* reasoning is not going to force
/// a break anyway. An earliest-first pass that stopped at the first profitable candidate would
/// break the prefix too early and pay for a re-prefill that a later break would have covered.
///
/// Reasoning belonging to `active_turn_id` is left undecided: the model is still following it,
/// and its final size is not known yet.
pub(crate) fn freeze_reasoning_retention(
    items: &mut [ResponseItemEnvelope],
    active_turn_id: Option<&str>,
    inputs: RetentionInputs,
) {
    // Where the prefix is already broken by something this pass does not control: an item from
    // a finished invisible turn, reasoning already sentenced to be dropped, or reasoning that
    // predates this metadata and is dropped for the same reason it always was.
    let forced_break = items
        .iter()
        .position(|envelope| is_dropped_regardless(envelope, active_turn_id))
        .unwrap_or(items.len());

    let mut candidates = Vec::new();
    let mut free_drops = Vec::new();
    for (index, envelope) in items.iter().enumerate() {
        if !is_undecided_completed_reasoning(envelope, active_turn_id) {
            continue;
        }
        if index >= forced_break {
            // Free: everything from `forced_break` on is re-prefilled either way.
            free_drops.push(index);
        } else {
            candidates.push((index, reasoning_tokens(&envelope.item)));
        }
    }
    for index in free_drops {
        stamp(&mut items[index], false);
    }
    if candidates.is_empty() {
        return;
    }

    // Tokens from each index to the end of history. Reasoning contributes nothing here, which
    // is why its size is measured separately.
    let mut suffix_tokens = vec![0i64; items.len() + 1];
    for (index, envelope) in items.iter().enumerate().rev() {
        suffix_tokens[index] =
            suffix_tokens[index + 1].saturating_add(estimate_item_token_count(&envelope.item));
    }

    let horizon = horizon(&inputs);
    let total: i64 = candidates
        .iter()
        .map(|(_, tokens)| *tokens)
        .fold(0i64, i64::saturating_add);

    // Start at the latest possible break, which keeps every candidate, then walk backwards
    // trading one more re-prefilled suffix against the reasoning that break drops.
    let mut best_break = forced_break;
    let mut best_cost = break_cost(suffix_tokens[forced_break], total, horizon);
    let mut dropped = 0i64;
    for (index, tokens) in candidates.iter().rev() {
        dropped = dropped.saturating_add(*tokens);
        let cost = break_cost(
            suffix_tokens[*index],
            total.saturating_sub(dropped),
            horizon,
        );
        if cost < best_cost {
            best_cost = cost;
            best_break = *index;
        }
    }

    for (index, _) in candidates {
        stamp(&mut items[index], index < best_break);
    }
}

fn stamp(envelope: &mut ResponseItemEnvelope, retained: bool) {
    envelope.metadata.get_or_insert_default().reasoning_retained = Some(retained);
}

/// Whether an item is removed from the prompt whatever this pass decides.
fn is_dropped_regardless(envelope: &ResponseItemEnvelope, active_turn_id: Option<&str>) -> bool {
    let metadata = envelope.metadata.as_ref();
    let invisible_owner = metadata.and_then(|metadata| metadata.invisible_turn.as_deref());
    if invisible_owner.is_some() && !belongs_to_active_turn(active_turn_id, invisible_owner) {
        return true;
    }
    if !matches!(envelope.item, ResponseItem::Reasoning { .. }) {
        return false;
    }
    let Some(metadata) = metadata else {
        // Reasoning recorded before this metadata existed. It has always been dropped.
        return true;
    };
    match metadata.reasoning_retained {
        Some(retained) => !retained,
        // An unstamped item with no owning turn cannot have been produced by this build.
        None => metadata.reasoning_turn.is_none(),
    }
}

fn is_undecided_completed_reasoning(
    envelope: &ResponseItemEnvelope,
    active_turn_id: Option<&str>,
) -> bool {
    if !matches!(envelope.item, ResponseItem::Reasoning { .. }) {
        return false;
    }
    let Some(metadata) = envelope.metadata.as_ref() else {
        return false;
    };
    metadata.reasoning_retained.is_none()
        && metadata.reasoning_turn.is_some()
        && !belongs_to_active_turn(active_turn_id, metadata.reasoning_turn.as_deref())
}

fn belongs_to_active_turn(active_turn_id: Option<&str>, owner: Option<&str>) -> bool {
    matches!((active_turn_id, owner), (Some(active), Some(owner)) if active == owner)
}

/// Size of the reasoning text that actually goes on the wire.
///
/// The history-wide estimator reports reasoning as zero tokens, because for most of this
/// codebase reasoning is something the server bills rather than something the prompt carries.
/// Here it is precisely the thing being priced, so it is measured directly.
fn reasoning_tokens(item: &ResponseItem) -> i64 {
    let ResponseItem::Reasoning {
        summary, content, ..
    } = item
    else {
        return 0;
    };
    let summary = summary.iter().map(|entry| match entry {
        ReasoningItemReasoningSummary::SummaryText { text } => text.as_str(),
    });
    let content = content.iter().flatten().map(|entry| match entry {
        ReasoningItemContent::ReasoningText { text } | ReasoningItemContent::Text { text } => {
            text.as_str()
        }
    });
    summary
        .chain(content)
        .map(|text| i64::try_from(approx_token_count(text)).unwrap_or(i64::MAX))
        .fold(0i64, i64::saturating_add)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_history::CodexHarnessMetadata;
    use codex_protocol::models::ContentItem;

    const TURN: &str = "turn-1";

    /// Roughly `tokens` tokens of history, so suffix sizes are readable in the tests.
    fn filler(tokens: usize) -> ResponseItemEnvelope {
        ResponseItemEnvelope::new(ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "x".repeat(tokens.saturating_mul(4)),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        })
    }

    fn thinking(tokens: usize, turn: &str) -> ResponseItemEnvelope {
        ResponseItemEnvelope {
            item: ResponseItem::Reasoning {
                id: None,
                summary: Vec::new(),
                content: Some(vec![ReasoningItemContent::ReasoningText {
                    text: "x".repeat(tokens.saturating_mul(4)),
                }]),
                encrypted_content: None,
                internal_chat_message_metadata_passthrough: None,
            },
            metadata: Some(CodexHarnessMetadata {
                reasoning_turn: Some(turn.to_string()),
                ..Default::default()
            }),
        }
    }

    fn retained(items: &[ResponseItemEnvelope]) -> Vec<Option<bool>> {
        items
            .iter()
            .filter(|envelope| matches!(envelope.item, ResponseItem::Reasoning { .. }))
            .map(|envelope| {
                envelope
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.reasoning_retained)
            })
            .collect()
    }

    /// A 40-request horizon: a full window of budget left, and 40 requests to a window.
    fn inputs() -> RetentionInputs {
        RetentionInputs {
            budget_remaining: WINDOW_TOKENS,
            requests_per_window: 40.0,
        }
    }

    #[test]
    fn tiny_reasoning_is_kept_because_dropping_it_rewrites_the_whole_turn() {
        let mut items = vec![thinking(32, TURN), filler(15_000)];

        freeze_reasoning_retention(&mut items, /*active_turn_id*/ None, inputs());

        assert_eq!(retained(&items), vec![Some(true)]);
    }

    #[test]
    fn large_reasoning_is_dropped_once_it_outweighs_the_reprefill() {
        let mut items = vec![thinking(5_000, TURN), filler(15_000)];

        freeze_reasoning_retention(&mut items, /*active_turn_id*/ None, inputs());

        assert_eq!(retained(&items), vec![Some(false)]);
    }

    #[test]
    fn later_reasoning_is_dropped_for_free_once_the_prefix_is_already_broken() {
        let mut items = vec![
            thinking(5_000, TURN),
            filler(15_000),
            thinking(32, "turn-2"),
            filler(10),
        ];

        freeze_reasoning_retention(&mut items, /*active_turn_id*/ None, inputs());

        assert_eq!(retained(&items), vec![Some(false), Some(false)]);
    }

    #[test]
    fn a_decision_is_never_revisited() {
        let mut items = vec![thinking(32, TURN), filler(15_000)];

        freeze_reasoning_retention(&mut items, /*active_turn_id*/ None, inputs());
        let frozen = retained(&items);
        freeze_reasoning_retention(
            &mut items,
            /*active_turn_id*/ None,
            RetentionInputs {
                budget_remaining: 0,
                requests_per_window: 1.0,
            },
        );

        assert_eq!(retained(&items), frozen);
    }

    #[test]
    fn active_turn_reasoning_is_left_undecided() {
        let mut items = vec![thinking(5_000, TURN), filler(15_000)];

        freeze_reasoning_retention(&mut items, Some(TURN), inputs());

        assert_eq!(retained(&items), vec![None]);
    }
}
