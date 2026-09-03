//! Renders a session as the model experienced it, next to what actually happened.
//!
//! This is an offline reader, not a second writer. The rollout already holds the untrimmed
//! truth: shrinking and dropping happen on the per-request copy of history, never on what is
//! persisted. The statistics sidecar holds what each of those requests cost and what building
//! it removed. Joining the two after the fact costs nothing at run time, cannot drift from what
//! shipped, and works on rollouts recorded before any of this existed.
//!
//! It reads two files and writes stdout. It builds no `Config` and needs no auth, so it can be
//! pointed at a rollout copied off another machine.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use codex_core::RequestStatsHeader;
use codex_core::RequestStatsLine;
use codex_history::RolloutItem;
use codex_history::RolloutLine;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::EventMsg;

/// Cached input as a fraction of uncached, so one request's bill is a single comparable number.
const CACHED_INPUT_PRICE_RATIO: f64 = 0.1;
/// How much of a tool output to quote before saying how much is left.
const QUOTED_OUTPUT_LINES: usize = 3;

/// The rollout items a single request produced, in the order the model emitted them.
struct Request {
    stats: Option<RequestStatsLine>,
    turn_id: Option<String>,
    items: Vec<ResponseItem>,
}

impl Request {
    /// Uncached-equivalent input, the one number a request can be compared on.
    fn effective_input(&self) -> f64 {
        self.stats.as_ref().map_or(0.0, |stats| {
            effective_input(stats.input_tokens, stats.cached_input_tokens)
        })
    }
}

fn effective_input(input: i64, cached: i64) -> f64 {
    (input - cached) as f64 + CACHED_INPUT_PRICE_RATIO * cached as f64
}

/// The whole session's bill according to the provider, which is the only total that is not the
/// log's own arithmetic. `total_token_usage` is cumulative, so the last one carries the session.
fn rollout_effective_input(lines: &[RolloutLine]) -> Option<f64> {
    lines
        .iter()
        .rev()
        .find_map(|line| match &line.item {
            RolloutItem::EventMsg(EventMsg::TokenCount(event)) => event.info.as_ref(),
            _ => None,
        })
        .map(|info| {
            effective_input(
                info.total_token_usage.input_tokens,
                info.total_token_usage.cached_input_tokens,
            )
        })
}

pub(crate) fn render(rollout: &Path, stats: Option<&Path>) -> Result<String> {
    let lines = read_rollout(rollout)?;
    let stats_path = stats
        .map(PathBuf::from)
        .or_else(|| default_stats_path(rollout));
    let (header, rows) = match stats_path.as_deref() {
        Some(path) if path.exists() => read_stats(path)?,
        _ => (None, Vec::new()),
    };

    let mut out = String::new();
    write_preamble(
        &mut out,
        rollout,
        stats_path.as_deref(),
        header.as_ref(),
        &lines,
    );
    let billed = rollout_effective_input(&lines);
    let requests = split_into_requests(lines, rows);
    write_requests(&mut out, &requests);
    write_trailer(&mut out, &requests, billed);
    Ok(out)
}

fn read_rollout(path: &Path) -> Result<Vec<RolloutLine>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read rollout {}", path.display()))?;
    // A rollout is appended to live, so a truncated final line is normal rather than an error.
    Ok(text
        .lines()
        .filter_map(|line| serde_json::from_str::<RolloutLine>(line).ok())
        .collect())
}

/// The sidecar is one header followed by one row per request, and a resumed session appends a
/// second header. Rows are what matter, so anything that is not one is skipped.
fn read_stats(path: &Path) -> Result<(Option<RequestStatsHeader>, Vec<RequestStatsLine>)> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read statistics {}", path.display()))?;
    let mut header = None;
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Ok(row) = serde_json::from_str::<RequestStatsLine>(line) {
            rows.push(row);
        } else if let Ok(parsed) = serde_json::from_str::<RequestStatsHeader>(line) {
            header.get_or_insert(parsed);
        }
    }
    Ok((header, rows))
}

/// `.../sessions/**/rollout-<date>-<thread-id>.jsonl` pairs with
/// `.../analytics/<thread-id>.jsonl`, which is why the sidecar is keyed on the thread id.
fn default_stats_path(rollout: &Path) -> Option<PathBuf> {
    // The thread id is a uuid, so it is the last five dash-separated groups of the stem.
    let groups: Vec<&str> = rollout.file_stem()?.to_str()?.split('-').collect();
    let thread_id = groups.get(groups.len().checked_sub(5)?..)?.join("-");
    let sessions = rollout
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "sessions"))?;
    Some(
        sessions
            .parent()?
            .join("analytics")
            .join(format!("{thread_id}.jsonl")),
    )
}

/// Splits the rollout into requests.
///
/// A `TokenCount` event is emitted once per completed request, which makes it the boundary: the
/// response items since the previous one are what that request produced. The sidecar rows are
/// written in the same order, so they line up positionally without needing a shared id.
fn split_into_requests(lines: Vec<RolloutLine>, rows: Vec<RequestStatsLine>) -> Vec<Request> {
    let mut requests = Vec::new();
    let mut rows = rows.into_iter();
    let mut items = Vec::new();
    let mut turn_id = None;
    for line in lines {
        match line.item {
            RolloutItem::ResponseItem(envelope) => items.push(envelope.item),
            RolloutItem::TurnContext(context) => turn_id = context.turn_id,
            RolloutItem::EventMsg(EventMsg::TokenCount(_)) => {
                requests.push(Request {
                    stats: rows.next(),
                    turn_id: turn_id.clone(),
                    items: std::mem::take(&mut items),
                });
            }
            _ => {}
        }
    }
    if !items.is_empty() {
        requests.push(Request {
            stats: rows.next(),
            turn_id,
            items,
        });
    }
    requests
}

fn write_preamble(
    out: &mut String,
    rollout: &Path,
    stats: Option<&Path>,
    header: Option<&RequestStatsHeader>,
    lines: &[RolloutLine],
) {
    let _ = writeln!(out, "# {}", rollout.display());
    match (stats, header) {
        (Some(path), Some(header)) => {
            let _ = writeln!(out, "statistics: {}", path.display());
            let _ = writeln!(
                out,
                "{} on {} at effort {} · suffice {} · features [{}] · retention priced at {:.0} requests per window",
                header.model,
                header.provider,
                header.reasoning_effort.as_deref().unwrap_or("default"),
                header.codex_version,
                header.features.join(", "),
                header.requests_per_window,
            );
        }
        _ => {
            let _ = writeln!(
                out,
                "statistics: none found. Only the rollout is shown; nothing below reports what \
                 building a prompt removed. Set `[features] request_stats = true` and run again."
            );
        }
    }
    if let Some(timestamp) = lines.first().map(|line| line.timestamp.as_str()) {
        let _ = writeln!(out, "starting {timestamp}");
    }
    let _ = writeln!(
        out,
        "\n> Caveat: on the Chat wire reasoning is not a message of its own. It is folded into \
         the message it explains as `reasoning_content`, and consecutive blocks are joined. \
         \"the model saw reasoning X\" means X rode along on that message."
    );
}

fn write_requests(out: &mut String, requests: &[Request]) {
    let mut current_turn = None;
    for (index, request) in requests.iter().enumerate() {
        if request.turn_id != current_turn {
            current_turn = request.turn_id.clone();
            write_turn_heading(out, requests, index);
        }
        let _ = writeln!(out, "\n### Request {index}");
        write_request_facts(out, request);
        write_items(out, request);
    }
}

fn write_turn_heading(out: &mut String, requests: &[Request], first: usize) {
    let turn: Vec<&Request> = requests[first..]
        .iter()
        .take_while(|request| request.turn_id == requests[first].turn_id)
        .collect();
    let effective: f64 = turn.iter().map(|request| request.effective_input()).sum();
    let (input, cached) = turn
        .iter()
        .filter_map(|request| request.stats.as_ref())
        .fold((0i64, 0i64), |(input, cached), stats| {
            (
                input + stats.input_tokens,
                cached + stats.cached_input_tokens,
            )
        });
    let _ = writeln!(
        out,
        "\n\n## Turn {} — requests {}-{} ({} requests, {} effective input, {})",
        requests[first].turn_id.as_deref().unwrap_or("?"),
        first,
        first + turn.len() - 1,
        turn.len(),
        thousands(effective as i64),
        percentage(cached, input),
    );
}

fn write_request_facts(out: &mut String, request: &Request) {
    let Some(stats) = request.stats.as_ref() else {
        let _ = writeln!(
            out,
            "no statistics row: this request predates the log, or the feature was off."
        );
        return;
    };
    match (stats.prompt_items, stats.prompt_tokens_estimated) {
        (Some(items), Some(tokens)) => {
            let _ = writeln!(
                out,
                "prompt: {items} items · {} est · {} tool specs · {}",
                thousands(tokens),
                stats.prompt_tool_specs.unwrap_or_default(),
                match stats.prefix_break {
                    // The cache is only paid for when the prefix stops matching, so this is the
                    // whole price of every filter that ran.
                    Some(at) => format!(
                        "cache broken at item {at} (re-prefill {})",
                        thousands(stats.prefix_break_tokens)
                    ),
                    None => "prefix unchanged, nothing re-prefilled".to_string(),
                }
            );
        }
        // Compaction builds its own prompt and never reports it.
        _ => {
            let _ = writeln!(out, "prompt: not measured (built outside the turn loop)");
        }
    }
    let _ = writeln!(
        out,
        "billed: {} input ({} cached) · {} output · {} reasoning · {} ms",
        thousands(stats.input_tokens),
        thousands(stats.cached_input_tokens),
        thousands(stats.output_tokens),
        thousands(stats.reasoning_output_tokens),
        stats.duration_ms,
    );

    let mut hidden = Vec::new();
    if stats.dropped_reasoning_items > 0 {
        hidden.push(format!(
            "reasoning from finished turns: {} items, {} tokens",
            stats.dropped_reasoning_items,
            thousands(stats.dropped_reasoning_tokens)
        ));
    }
    if stats.retained_reasoning_items > 0 {
        hidden.push(format!(
            "reasoning kept and re-sent: {} items, {} tokens (priced against a horizon of {})",
            stats.retained_reasoning_items,
            thousands(stats.retained_reasoning_tokens),
            stats
                .retention_horizon
                .map_or_else(|| "no verdict".to_string(), |horizon| format!("{horizon}"))
        ));
    }
    if stats.dropped_invisible_items > 0 {
        hidden.push(format!(
            "items of a finished invisible turn: {} items, {} tokens{}",
            stats.dropped_invisible_items,
            thousands(stats.dropped_invisible_tokens),
            join_ids(" for ", &stats.dropped_call_ids),
        ));
    }
    if stats.shrunk_outputs > 0 {
        hidden.push(format!(
            "{} tool outputs trimmed, {} lines and {} tokens removed{}",
            stats.shrunk_outputs,
            stats.shrunk_lines_removed,
            thousands(stats.shrunk_tokens_removed),
            join_ids(": ", &stats.shrunk_call_ids),
        ));
    }
    if stats.shrinkable_before_break > 0 {
        // The saving the cache gate declined to take. A column of these and no `shrunk_outputs`
        // is the shrinker being dormant rather than the allowlist being too narrow.
        hidden.push(format!(
            "{} tool outputs were trimmable but sat before the break, so were left whole",
            stats.shrinkable_before_break
        ));
    }
    if hidden.is_empty() {
        let _ = writeln!(out, "the model saw the whole history");
    } else {
        let _ = writeln!(out, "hidden from the model at this point:");
        for entry in hidden {
            let _ = writeln!(out, "  - {entry}");
        }
    }
    if stats.tool_calls > 0 {
        let _ = writeln!(
            out,
            "model replied with {} tool call{}{}",
            stats.tool_calls,
            if stats.tool_calls == 1 { "" } else { "s" },
            if stats.tool_calls > 1 {
                " (batched)"
            } else {
                ""
            },
        );
    }
}

fn write_items(out: &mut String, request: &Request) {
    let shrunk = request
        .stats
        .as_ref()
        .map(|stats| stats.shrunk_call_ids.as_slice())
        .unwrap_or_default();
    for item in &request.items {
        match item {
            ResponseItem::Message { role, content, .. } => {
                let _ = writeln!(out, "\n> {role}: {}", indent(&message_text(content)));
            }
            ResponseItem::Reasoning { .. } => {
                let _ = writeln!(out, "\n> reasoning (folded into the message below)");
            }
            ResponseItem::FunctionCall {
                name,
                arguments,
                call_id,
                ..
            } => {
                let _ = writeln!(
                    out,
                    "\n> tool_call {name} {call_id}: {}",
                    first_line(arguments)
                );
            }
            ResponseItem::FunctionCallOutput {
                call_id, output, ..
            } => {
                let FunctionCallOutputBody::Text(text) = &output.body else {
                    continue;
                };
                let call_id = call_id.as_deref().unwrap_or("?");
                let was_shrunk = shrunk.iter().any(|shrunk| shrunk == call_id);
                let total = text.lines().count();
                let _ = writeln!(
                    out,
                    "\n> tool_output {call_id} — {}",
                    if was_shrunk {
                        // The rollout keeps the untrimmed text, so this is what actually
                        // happened; the model was handed the head and the tail of it.
                        format!(
                            "WHAT ACTUALLY HAPPENED ({total} lines, the model saw a head and a tail of it)"
                        )
                    } else {
                        format!("{total} lines, seen whole")
                    }
                );
                let _ = writeln!(out, "{}", quote(text));
            }
            _ => {}
        }
    }
}

fn write_trailer(out: &mut String, requests: &[Request], billed: Option<f64>) {
    let rows: Vec<&RequestStatsLine> = requests
        .iter()
        .filter_map(|request| request.stats.as_ref())
        .collect();
    if rows.is_empty() {
        return;
    }
    let bill: f64 = requests.iter().map(Request::effective_input).sum();
    let _ = writeln!(out, "\n\n## What the optimisations were worth");
    let _ = writeln!(
        out,
        "\n{} requests · effective input B = {} · {}",
        rows.len(),
        thousands(bill as i64),
        percentage(
            rows.iter().map(|row| row.cached_input_tokens).sum(),
            rows.iter().map(|row| row.input_tokens).sum(),
        ),
    );

    // A retained token is re-billed on every later request until compaction rewrites the prefix
    // wholesale, which is what makes a drop worth its re-prefill. Total tokens falling is the
    // compaction event, so it is also the window boundary.
    let remaining = requests_until_compaction(&rows);
    let mut retention_saved = 0.0;
    let mut retention_cost = 0.0;
    let mut standing_keep = 0.0;
    let mut shrinker = 0.0;
    for (index, row) in rows.iter().enumerate() {
        let horizon = remaining[index] as f64;
        retention_saved += CACHED_INPUT_PRICE_RATIO * row.dropped_reasoning_tokens as f64 * horizon;
        retention_cost += (1.0 - CACHED_INPUT_PRICE_RATIO) * row.prefix_break_tokens as f64;
        standing_keep += CACHED_INPUT_PRICE_RATIO * row.retained_reasoning_tokens as f64;
        shrinker += CACHED_INPUT_PRICE_RATIO * row.shrunk_tokens_removed as f64 * horizon;
    }
    let invisible: f64 = requests
        .iter()
        .filter(|request| request.stats.as_ref().is_some_and(|stats| stats.invisible))
        .map(Request::effective_input)
        .sum();
    // A request that reports no prompt was built outside the turn loop, which is the compaction
    // summariser. It runs on a prefix of its own, so it is never cached and never discounted.
    let compaction: f64 = requests
        .iter()
        .filter(|request| {
            request
                .stats
                .as_ref()
                .is_some_and(|stats| stats.prompt_items.is_none())
        })
        .map(Request::effective_input)
        .sum();

    let _ = writeln!(
        out,
        "\nreasoning retention:  {:+}",
        (retention_saved - retention_cost) as i64
    );
    let _ = writeln!(
        out,
        "  saved by dropping:  {}",
        thousands(retention_saved as i64)
    );
    let _ = writeln!(
        out,
        "  paid in re-prefill: {}",
        thousands(retention_cost as i64)
    );
    let _ = writeln!(
        out,
        "standing keep cost:   {}",
        thousands(standing_keep as i64)
    );
    let _ = writeln!(
        out,
        "shrinker:             {} (costs nothing; it only ever runs past a break someone else paid for)",
        thousands(shrinker as i64)
    );
    let _ = writeln!(
        out,
        "invisible turns:      {} spent",
        thousands(invisible as i64)
    );
    let _ = writeln!(
        out,
        "compaction summaries: {} spent",
        thousands(compaction as i64)
    );
    // The one line the log cannot fake: everything above is the log's own arithmetic, this is the
    // provider's. A large residue means requests are being billed that no row accounts for, so
    // every share above is measured against the wrong denominator.
    match billed {
        Some(billed) => {
            let _ = writeln!(
                out,
                "unattributed:         {} of {} the provider billed ({})",
                thousands((billed - bill) as i64),
                thousands(billed as i64),
                percentage_of(billed - bill, billed),
            );
        }
        None => {
            let _ = writeln!(
                out,
                "unattributed:         unknown, the rollout carries no token count"
            );
        }
    }

    let (unexplained, unexplained_rows) = unexplained_reprefill(&rows);
    if unexplained > 0.0 {
        // The filters are not the only thing that rewrites a prompt. Anything re-injected at the
        // start of a turn moves the prefix too, and the report cannot see it because no filter
        // ran. This is the gap between what the log says it removed and what the bill says.
        let _ = writeln!(
            out,
            "\n{unexplained_rows} requests reported an untouched prefix and were charged full \
             price anyway, {} of it. No filter did that: something outside history rewrote the \
             prompt, which at a turn boundary is the harness re-injecting its own preamble.",
            thousands(unexplained as i64),
        );
    }

    let unarmed = rows.iter().filter(|row| row.prompt_items.is_none()).count();
    let unbroken = rows.iter().filter(|row| row.prefix_break.is_none()).count();
    let declined: usize = rows.iter().map(|row| row.shrinkable_before_break).sum();
    let _ = writeln!(
        out,
        "\n{unbroken} of {} requests left the prefix untouched. The shrinker cannot run on those, \
         and it declined {declined} trimmable outputs in total.",
        rows.len()
    );
    if unarmed > 0 {
        let _ = writeln!(
            out,
            "{unarmed} requests report no prompt: they were built outside the turn loop, which \
             is compaction. Their bill is counted, their contents are not attributed."
        );
    }
    let calls: usize = rows.iter().map(|row| row.tool_calls).sum();
    let replies = rows.iter().filter(|row| row.tool_calls > 0).count();
    if replies > 0 {
        let _ = writeln!(
            out,
            "{calls} tool calls in {replies} replies = {:.2} per reply.",
            calls as f64 / replies as f64
        );
    }
    let specs: BTreeMap<usize, usize> = rows.iter().filter_map(|row| row.prompt_tool_specs).fold(
        BTreeMap::new(),
        |mut counts, specs| {
            *counts.entry(specs).or_default() += 1;
            counts
        },
    );
    if specs.len() > 1 {
        // The tool list is part of the fixed prefix, so a change to it rewrites every prompt
        // from its first token. An MCP server connecting mid-session does exactly that.
        let _ = writeln!(
            out,
            "WARNING: the tool list changed mid-session ({}), which rewrites the prefix from its \
             first token every time it happens.",
            specs
                .iter()
                .map(|(specs, count)| format!("{specs} specs x{count}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// Full-price input that no reported filter accounts for.
///
/// A request whose prefix the report left alone should have been served everything the previous
/// request sent. Whatever it was charged for beyond that was re-prefilled by something the report
/// cannot see. Compaction is excluded, since it rewrites the prefix on purpose and shows up as
/// total tokens falling.
fn unexplained_reprefill(rows: &[&RequestStatsLine]) -> (f64, usize) {
    let mut tokens = 0.0;
    let mut count = 0;
    for (previous, row) in rows.iter().zip(rows.iter().skip(1)) {
        if row.prefix_break.is_some() || row.total_tokens < previous.total_tokens {
            continue;
        }
        let missing = previous.input_tokens - row.cached_input_tokens;
        if missing > 0 {
            tokens += (1.0 - CACHED_INPUT_PRICE_RATIO) * missing as f64;
            count += 1;
        }
    }
    (tokens, count)
}

/// How many later requests share this one's prefix, i.e. how many times a token kept now is
/// re-billed. Compaction rewrites the prefix wholesale, and shows up as total tokens falling.
fn requests_until_compaction(rows: &[&RequestStatsLine]) -> Vec<usize> {
    let mut remaining = vec![0; rows.len()];
    for (index, count) in remaining.iter_mut().enumerate() {
        *count = rows[index + 1..]
            .iter()
            .zip(&rows[index..])
            .take_while(|(next, previous)| next.total_tokens >= previous.total_tokens)
            .count();
    }
    remaining
}

/// Concatenates, because a streamed message is persisted as one text part per token and the wire
/// puts them back together with nothing between them.
fn message_text(content: &[ContentItem]) -> String {
    content
        .iter()
        .filter_map(|item| match item {
            ContentItem::InputText { text } | ContentItem::OutputText { text, .. } => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect()
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or_default()
}

fn indent(text: &str) -> String {
    text.replace('\n', "\n> ")
}

fn quote(text: &str) -> String {
    let mut lines: Vec<String> = text
        .lines()
        .take(QUOTED_OUTPUT_LINES)
        .map(|line| format!(">   {line}"))
        .collect();
    let total = text.lines().count();
    if total > QUOTED_OUTPUT_LINES {
        lines.push(format!(
            ">   ... {} more lines",
            total - QUOTED_OUTPUT_LINES
        ));
    }
    lines.join("\n")
}

fn join_ids(prefix: &str, ids: &[String]) -> String {
    if ids.is_empty() {
        String::new()
    } else {
        format!("{prefix}{}", ids.join(", "))
    }
}

fn percentage_of(part: f64, whole: f64) -> String {
    if whole <= 0.0 {
        return "no bill".to_string();
    }
    format!("{:.1}%", 100.0 * part / whole)
}

fn percentage(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "no input".to_string();
    }
    format!("{:.1}% cached", 100.0 * part as f64 / whole as f64)
}

fn thousands(tokens: i64) -> String {
    if tokens.abs() >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
