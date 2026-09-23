//! Fits an `exec` result into its budget without cutting blind.
//!
//! Upstream joins every text item a script produced into one string and cuts the middle out of
//! it. That is the wrong unit twice over. A script's answer is usually a short `text()` next to a
//! long dump, and a middle cut through the joined string can take the answer along with the dump;
//! and what it cut is gone, where the direct `exec_command` path writes the rest to a file the
//! model can read back.
//!
//! Three passes, in order:
//!
//! 1. An item that is, byte for byte, the output of a nested `exec_command` the same cell ran is
//!    condensed the way that command's output is condensed on the direct path - the command and
//!    its exit code are known, so the RTK-derived stages can run. Only verbatim echoes qualify:
//!    anything the script computed itself is its answer and is never condensed.
//! 2. The budget is shared per item, smallest first. Items that fit their share stay whole; only
//!    the largest are cut, and each by the same amount.
//! 3. Whatever a pass removed is written to a file and the item names its path, so a cut is
//!    recoverable rather than final.
//!
//! Nothing here runs unless code mode itself does, and code mode only runs when its host
//! executable is installed.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use codex_protocol::models::FunctionCallOutputContentItem;
use codex_utils_output_truncation::TruncationPolicy;
use codex_utils_output_truncation::formatted_truncate_text;
use codex_utils_output_truncation::never_worse;

use crate::context_manager::tool_output::condense_exec_output;

/// How many nested command results one cell keeps for matching.
///
/// A cell that runs more commands than this is looping over something, and the items it prints
/// are then far more likely to be its own summaries than raw dumps. The oldest are dropped first.
const MAX_RECORDS_PER_CELL: usize = 32;

/// A command output larger than this is not kept for matching.
///
/// Matching is an equality test, so keeping the text costs memory for as long as the cell lives;
/// an output this large would be cut to a fraction of itself by pass 2 regardless.
const MAX_RECORD_BYTES: usize = 1024 * 1024;

/// Mirrors `SPILL_NOTICE_BUDGET_RATIO` in `tools/context.rs`: a path is only worth naming when
/// the share it is taken from is at least this many times its length.
const SPILL_NOTICE_BUDGET_RATIO: usize = 8;

/// One nested `exec_command` (or `write_stdin`) result, as the script received it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NestedExecRecord {
    /// The call's JSON arguments, which is what `condense_exec_output` reads the command from.
    pub(crate) arguments: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) process_id: Option<i32>,
    /// The `output` field the script was handed: lossless-condensed, not yet trimmed.
    pub(crate) output: String,
}

/// Nested command results per running cell, so the cell's final output can be matched against
/// them.
#[derive(Default)]
pub(crate) struct NestedExecLedger {
    cells: Mutex<HashMap<String, VecDeque<NestedExecRecord>>>,
}

impl NestedExecLedger {
    pub(crate) fn record(&self, cell_id: &str, record: NestedExecRecord) {
        if record.output.len() > MAX_RECORD_BYTES || record.output.is_empty() {
            return;
        }
        let Ok(mut cells) = self.cells.lock() else {
            return;
        };
        let records = cells.entry(cell_id.to_string()).or_default();
        if records.len() == MAX_RECORDS_PER_CELL {
            records.pop_front();
        }
        records.push_back(record);
    }

    /// The cell's records. A cell that has finished is forgotten; a yielded one keeps its records
    /// for the `wait` that will return the rest of its output.
    pub(crate) fn records(&self, cell_id: &str, finished: bool) -> Vec<NestedExecRecord> {
        let Ok(mut cells) = self.cells.lock() else {
            return Vec::new();
        };
        if finished {
            cells
                .remove(cell_id)
                .map(Vec::from)
                .unwrap_or_default()
        } else {
            cells
                .get(cell_id)
                .map(|records| records.iter().cloned().collect())
                .unwrap_or_default()
        }
    }
}

/// Where a cut item's full text goes, and what to call the file.
#[derive(Debug, Clone)]
pub(crate) struct SpillTarget {
    pub(crate) dir: PathBuf,
    /// Unique per response: the `exec` or `wait` call id.
    pub(crate) stem: String,
}

impl SpillTarget {
    fn path(&self, index: usize) -> PathBuf {
        let mut stem: String = self
            .stem
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if stem.is_empty() {
            stem.push_str("exec");
        }
        // `.txt` because `read` refuses the extensions it treats as binary.
        self.dir.join(format!("code_mode_{stem}_{index}.txt"))
    }
}

fn spill_notice(path: &Path) -> String {
    format!("Full output saved to: {}", path.display())
}

/// Writes `text`, returning the path only if the write succeeded. A failed write drops the notice
/// and leaves the item as it would have been without this mechanism.
fn write_spill(path: &Path, text: &str) -> Option<PathBuf> {
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(path, text).ok()?;
    Some(path.to_path_buf())
}

/// A text item on its way through the passes: what the model will see, and the full text a
/// spill file should hold if anything was removed from it.
struct Shaped {
    text: String,
    original: Option<String>,
}

/// Applies all three passes. Items that are not all text keep upstream's handling, which already
/// budgets media and text together; `None` tells the caller to use it.
pub(crate) fn shape_text_items(
    items: &[FunctionCallOutputContentItem],
    records: &[NestedExecRecord],
    policy: TruncationPolicy,
    spill: Option<&SpillTarget>,
) -> Option<Vec<FunctionCallOutputContentItem>> {
    let mut shaped = Vec::with_capacity(items.len());
    for item in items {
        let FunctionCallOutputContentItem::InputText { text } = item else {
            return None;
        };
        shaped.push(condense_echo(text, records));
    }

    let budget = policy.byte_budget();
    let shares = fair_shares(
        &shaped.iter().map(|item| item.text.len()).collect::<Vec<_>>(),
        budget,
    );

    let items = shaped
        .into_iter()
        .zip(shares)
        .enumerate()
        .map(|(index, (Shaped { text, original }, share))| {
            let spill_path = spill.map(|target| target.path(index));
            match share {
                // Fits whole. Pass 1 may still have condensed it, and then what it dropped is
                // worth a file for the same reason a lossy stage spills on the direct path.
                None => match original {
                    Some(full) => finish(text, spill_path.as_deref(), &full),
                    None => FunctionCallOutputContentItem::InputText { text },
                },
                Some(share) => {
                    // The notice's room comes out of the item's own share, and only when the share
                    // is large enough that naming a file does not crowd out the text it recovers.
                    let reserved = spill_path
                        .as_deref()
                        .map(|path| spill_notice(path).len() + 1)
                        .filter(|reserved| reserved * SPILL_NOTICE_BUDGET_RATIO <= share)
                        .unwrap_or(0);
                    let cut = formatted_truncate_text(&text, in_unit_of(policy, share - reserved));
                    // The file holds what the script printed, before either pass touched it.
                    let full = original.unwrap_or(text);
                    finish(cut, spill_path.filter(|_| reserved > 0).as_deref(), &full)
                }
            }
        })
        .collect();
    Some(items)
}

/// `bytes` expressed in `policy`'s unit, which decides whether the marker left in the text reads
/// `tokens truncated` or `chars truncated`.
fn in_unit_of(policy: TruncationPolicy, bytes: usize) -> TruncationPolicy {
    match policy {
        TruncationPolicy::Bytes(_) => TruncationPolicy::Bytes(bytes),
        TruncationPolicy::Tokens(_) => {
            TruncationPolicy::Tokens(TruncationPolicy::Bytes(bytes).token_budget())
        }
    }
}

/// Prefixes the spill notice when the full text could be saved.
fn finish(text: String, path: Option<&Path>, full: &str) -> FunctionCallOutputContentItem {
    let text = match path.and_then(|path| write_spill(path, full)) {
        Some(path) => format!("{}\n{text}", spill_notice(&path)),
        None => text,
    };
    FunctionCallOutputContentItem::InputText { text }
}

/// Pass 1: condenses an item that repeats a nested command's output verbatim.
fn condense_echo(text: &str, records: &[NestedExecRecord]) -> Shaped {
    let unchanged = || Shaped {
        text: text.to_string(),
        original: None,
    };
    // The most recent match wins: a cell that ran the same command twice echoes the later run.
    let Some(record) = records.iter().rev().find(|record| record.output == text) else {
        return unchanged();
    };
    let Some((condensed, report)) = condense_exec_output(
        &record.arguments,
        record.exit_code,
        record.process_id,
        text,
    ) else {
        return unchanged();
    };
    // Only a lossy stage is worth announcing, and only a lossy stage leaves anything to recover.
    // The lossless ones already ran before the script saw this text.
    let Some(summary) = report.summary() else {
        return unchanged();
    };
    let framed = format!("{summary}\n{condensed}");
    // `condense_exec_output` already requires a lossy stage to clear `MIN_LOSSY_TOKENS`, but the
    // summary line is added here, after that check; this is the check that includes it.
    if std::ptr::eq(never_worse(text, &framed), text) {
        return unchanged();
    }
    Shaped {
        text: framed,
        original: Some(text.to_string()),
    }
}

/// Pass 2: each item's share of `budget`, or `None` when it fits whole.
///
/// Max-min fair: items are taken smallest first, each against an equal split of what is left;
/// one that fits is kept whole and returns its unused share to the rest. The first item that does
/// not fit fixes the share for itself and every larger item.
fn fair_shares(sizes: &[usize], budget: usize) -> Vec<Option<usize>> {
    let total: usize = sizes.iter().sum();
    // Items are joined with newlines downstream of this; count them so a fit here is a fit there.
    let separators = sizes.len().saturating_sub(1);
    if total + separators <= budget {
        return vec![None; sizes.len()];
    }

    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&index| sizes[index]);

    let mut shares = vec![None; sizes.len()];
    let mut remaining = budget.saturating_sub(separators);
    for (position, &index) in order.iter().enumerate() {
        let left = order.len() - position;
        let share = remaining / left;
        if sizes[index] <= share {
            remaining -= sizes[index];
            continue;
        }
        for &cut in &order[position..] {
            shares[cut] = Some(share);
        }
        break;
    }
    shares
}

#[cfg(test)]
#[path = "result_budget_tests.rs"]
mod tests;
