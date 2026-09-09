//! Read many files in one round trip.
//!
//! Reading through the shell works, but the model has to express a batch as one chained command
//! (`Get-Content a; '---'; Get-Content b; ...`) whose *combined* output is capped, so batches stop
//! at a handful of files and the rest of the corpus costs a round trip each. Measured on a task that
//! reads 44 files: eight read turns here against three for an agent whose read tool takes a list.
//! Every extra turn resends the whole window, and enough of them reach the auto-compaction limit,
//! which destroys the tool output that was just paid for.
//!
//! An entry is a path or `{path, offset, limit}`, so one call can mix whole files with windows.
//! That matters more than the list does: the first version of this tool applied a window only when
//! exactly one path was passed and dropped it silently otherwise, so a model that wanted a window
//! had no way to ask for one inside a batch and fell back to reading one file per call. Every
//! single-path call in the recorded runs carried an `offset` - the model was not forgetting to
//! batch, it was chasing windows through the only door that was open.
//!
//! The call budget is spent max-min fair across the batch rather than first-come-first-served, so
//! twenty files each come back with a usable slice instead of three files eating everything and
//! seventeen being named as unread. The budget is enforced while assembling, not while emitting, so
//! the bound holds even for a batch that is entirely errors.

use codex_exec_server::FileMetadata;
use codex_exec_server::GetMetadataOptions;
use codex_exec_server::ReadDirectoryEntry;
use codex_exec_server::ReadFileOptions;
use codex_protocol::protocol::TruncationPolicy;
use codex_utils_path_uri::PathUri;
use futures::StreamExt;
use serde_json::Map;
use serde_json::Value;
use std::io;

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::read_spec::DEFAULT_LINE_LIMIT;
use crate::tools::handlers::read_spec::READ_TOOL_NAME;
use crate::tools::handlers::read_spec::ReadToolOptions;
use crate::tools::handlers::read_spec::create_read_tool;
use crate::tools::handlers::resolve_tool_environment;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

/// Ceiling and floor for one call's output, in bytes.
///
/// The real budget is derived per model by `call_budget`, because the harness cuts a
/// `function_call_output` middle-out at `truncation_policy * 1.2` and that limit is a property of
/// the model, not a constant. A fixed 96_000 is right for a 25k-token policy and roughly double
/// what a 10k-token model allows, where it would produce exactly the middle-out cut through the
/// middle of a file that this tool exists to avoid.
const MAX_CALL_BUDGET_BYTES: usize = 96_000;
const MIN_CALL_BUDGET_BYTES: usize = 8_000;
/// Share of the harness's own cut this tool will fill, leaving room for the header the harness adds
/// and for the estimate in `byte_budget` being approximate.
const BUDGET_HEADROOM_NUMERATOR: usize = 4;
const BUDGET_HEADROOM_DENOMINATOR: usize = 5;
/// Share of the auto-compaction window one tool output may fill.
///
/// The harness's own two knobs disagree: `truncation_policy` lets a single `function_call_output`
/// be 25k tokens while `auto_compact_token_limit` makes the whole window 80k, so four reads can
/// fill it. Measured on a 44-file task: one call returned 93,538 bytes, 29% of the window, and the
/// runs that did that compacted twice and cost 2.4x the run that did not. Compaction destroys the
/// tool output that was just paid for, so the window, not the harness cut, is the binding limit.
const WINDOW_SHARE_DENOMINATOR: usize = 6;
/// Ceiling for a single file, so one large file cannot turn a twenty-file batch into a one-file one.
const PER_FILE_BUDGET_BYTES: usize = 32_000;
/// Below this a slice is too small to answer anything, so the file is named as unread instead.
const MIN_FILE_BUDGET_BYTES: usize = 1_024;
/// A single line longer than this is almost always minified or generated. Bytes, not chars: the
/// budget is in bytes, and 2_000 chars of CJK is 6_000 of them.
const MAX_LINE_BYTES: usize = 2_000;
/// What a file's line numbers add to its output, as a share of the file's own size plus a floor.
///
/// The share is not constant: at ~35 bytes a line a `123: ` prefix is under a tenth, but a file of
/// six-byte lines pays half again its own size, and a two-line file pays more than half. The
/// estimate only decides how much budget to reserve - the body is bounded by the cap either way -
/// so it is biased generous: reserving a little too much wastes budget, reserving too little
/// truncates every file by exactly the numbering.
const NUMBERING_OVERHEAD_DIVISOR: usize = 2;
const NUMBERING_OVERHEAD_FLOOR_BYTES: usize = 64;
/// Entries past this are named in the trailer rather than read.
const MAX_ENTRIES: usize = 64;
/// A file larger than this is refused from its metadata, without reading a byte.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// Head sample used to decide whether a file is binary.
const BINARY_SAMPLE_BYTES: usize = 4_096;
/// Share of control bytes in the sample above which the file is treated as binary.
const BINARY_CONTROL_PERCENT: usize = 30;
/// Room reserved in each header for the note describing how the file was cut.
const NOTE_RESERVE_BYTES: usize = 96;
/// Room reserved for the closing trailer, so naming unread files can never break the bound.
const TRAILER_RESERVE_BYTES: usize = 1_600;
const TRAILER_MAX_NAMES: usize = 10;
const TRAILER_NAME_MAX_BYTES: usize = 120;
/// Bounds on concurrent filesystem work inside one call. Several `read` calls can be in flight at
/// once, so these multiply; keeping the read bound at 8 matches what other agents allow per turn.
const MAX_CONCURRENT_METADATA: usize = 32;
const MAX_CONCURRENT_READS: usize = 8;
const MAX_DIRECTORY_ENTRIES: usize = 40;
const DIRECTORY_BUDGET_BYTES: usize = 1_500;

const LIST_KEYS: &[&str] = &["paths", "files", "file_paths", "filePaths"];
const PATH_KEYS: &[&str] = &["path", "file_path", "filePath", "file"];
const OFFSET_KEYS: &[&str] = &["offset", "start_line", "startLine"];
const END_KEYS: &[&str] = &["end_line", "endLine"];

/// Extensions whose contents are never worth decoding as text.
const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "tiff", "pdf", "zip", "gz", "tgz", "tar",
    "bz2", "xz", "7z", "rar", "exe", "dll", "so", "dylib", "a", "o", "obj", "class", "jar", "wasm",
    "mp3", "mp4", "wav", "avi", "mov", "mkv", "flac", "ogg", "woff", "woff2", "ttf", "otf", "eot",
    "db", "sqlite", "sqlite3", "pyc", "pack", "idx", "bin", "dat",
];

#[derive(Default)]
pub struct ReadHandler {
    options: ReadToolOptions,
}

impl ReadHandler {
    pub(crate) fn new(options: ReadToolOptions) -> Self {
        Self { options }
    }
}

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

/// One file plus the window the caller asked for. Windows are 1-indexed and inclusive of `offset`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReadTarget {
    /// The path exactly as the model spelled it, so errors and the trailer name it back the way it
    /// was sent and a retry can copy the string verbatim.
    raw: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct NormalizedArgs {
    targets: Vec<ReadTarget>,
    environment_id: Option<String>,
    /// Entries past `MAX_ENTRIES`, named in the trailer rather than dropped in silence.
    overflow: Vec<String>,
}

/// The largest char boundary at or below `max`, so clipping never splits a code point.
fn clip_to_bytes(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn as_usize(value: &Value) -> Option<usize> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|number| usize::try_from(number).ok()),
        // A model that quotes its numbers means the same thing it would have meant unquoted.
        Value::String(text) => text.trim().parse::<usize>().ok(),
        _ => None,
    }
}

fn pick<'a>(object: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| object.get(*key))
}

/// `end_line` is inclusive, so lines 40 through 80 is 41 lines.
fn span(offset: Option<usize>, end: Option<usize>) -> Option<usize> {
    match (offset, end) {
        (Some(start), Some(end)) if end >= start => Some(end - start + 1),
        (None, Some(end)) => Some(end),
        _ => None,
    }
}

/// Teach the shape rather than quoting serde: "data did not match any variant of untagged enum"
/// tells a model nothing it can act on.
fn shape_error(arguments: &str) -> FunctionCallError {
    FunctionCallError::RespondToModel(format!(
        "read expects {{\"filePath\": \"other.rs\", \"offset\": 40, \"limit\": 80}}; got: {}",
        clip_to_bytes(arguments, 200)
    ))
}

/// Accept every shape a model plausibly emits and reduce it to one list of targets.
///
/// None of this appears in the schema, so it costs nothing on the fixed prefix that every request
/// carries. The alternative - a `serde(untagged)` enum - costs the same prefix and answers a
/// malformed call with a parser message instead of an example.
fn normalize_args(arguments: &str) -> Result<NormalizedArgs, FunctionCallError> {
    let mut value: Value = serde_json::from_str(arguments).map_err(|_| shape_error(arguments))?;
    // Arguments that arrive double-encoded are a JSON string holding the real object.
    if let Value::String(inner) = &value
        && let Ok(parsed) = serde_json::from_str::<Value>(inner)
    {
        value = parsed;
    }
    let object = value.as_object().ok_or_else(|| shape_error(arguments))?;

    let environment_id = pick(object, &["environment_id", "environmentId"])
        .and_then(Value::as_str)
        .map(str::to_string);

    let top_offset = pick(object, OFFSET_KEYS).and_then(as_usize);
    let top_limit = pick(object, &["limit"])
        .and_then(as_usize)
        .or_else(|| span(top_offset, pick(object, END_KEYS).and_then(as_usize)));

    let mut entries: Vec<Value> = Vec::new();
    match pick(object, LIST_KEYS) {
        Some(Value::Array(items)) => entries.extend(items.iter().cloned()),
        // A list key holding a single value means the same as a one-element list.
        Some(other @ (Value::String(_) | Value::Object(_))) => entries.push(other.clone()),
        _ => {}
    }
    if entries.is_empty()
        && let Some(single) = pick(object, PATH_KEYS)
    {
        entries.push(single.clone());
    }

    let mut targets: Vec<ReadTarget> = Vec::new();
    for entry in entries {
        match entry {
            Value::String(path) => {
                let path = path.trim().to_string();
                if !path.is_empty() {
                    targets.push(ReadTarget {
                        raw: path,
                        offset: None,
                        limit: None,
                    });
                }
            }
            Value::Object(map) => {
                let offset = pick(&map, OFFSET_KEYS).and_then(as_usize);
                let limit = pick(&map, &["limit"])
                    .and_then(as_usize)
                    .or_else(|| span(offset, pick(&map, END_KEYS).and_then(as_usize)));
                match pick(&map, PATH_KEYS)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|path| !path.is_empty())
                {
                    Some(path) => targets.push(ReadTarget {
                        raw: path.to_string(),
                        offset,
                        limit,
                    }),
                    // A range with no path of its own belongs to the entry before it: models split
                    // `{path, offset, limit}` into two array elements often enough that folding it
                    // back beats failing the whole call.
                    None => {
                        if let Some(previous) = targets.last_mut() {
                            if offset.is_some() {
                                previous.offset = offset;
                            }
                            if limit.is_some() {
                                previous.limit = limit;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // A top-level window is a default, not an override: an entry that asked for its own keeps it.
    //
    // It is replicated, not divided. Dividing it was measured and reverted: `limit: 400` across
    // eleven paths became 36 lines each, clamped up to a 40-line floor, so the model received a
    // tenth of what it asked for and each section closed with `continue with offset 41`. Over one
    // run that produced 83 invitations to come back, 37 read calls and 122 entries for a 44-file
    // corpus. Fulfilment tracks cost directly: 84% fulfilled cost $0.0241, 43% cost $0.0428. Giving
    // the model less than it asked for does not save the tokens; it defers them and adds a round
    // trip. The call budget already bounds the output - that bound is where the saving belongs.
    for target in &mut targets {
        if target.offset.is_none() && target.limit.is_none() {
            target.offset = top_offset;
            target.limit = top_limit;
        }
    }

    if targets.is_empty() {
        return Err(shape_error(arguments));
    }

    let overflow = if targets.len() > MAX_ENTRIES {
        targets
            .split_off(MAX_ENTRIES)
            .into_iter()
            .map(|target| target.raw)
            .collect()
    } else {
        Vec::new()
    };

    Ok(NormalizedArgs {
        targets,
        environment_id,
        overflow,
    })
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// One file's contribution to the output: a header, an optional note about how it was cut, a body.
#[derive(Debug, PartialEq, Eq)]
struct Section {
    display: String,
    note: Option<String>,
    body: String,
}

impl Section {
    fn note_only(display: String, reason: &str) -> Self {
        Self {
            display,
            note: None,
            body: format!("{reason}\n"),
        }
    }

    /// Bytes this section adds to the payload, header and trailing separator included.
    fn len(&self) -> usize {
        // "===== " + display + optional " (note)" + "\n" + body + "\n"
        "===== ".len()
            + self.display.len()
            + self.note.as_ref().map_or(0, |note| note.len() + 3)
            + 1
            + self.body.len()
            + 1
    }
}

/// Bytes a section can occupy before its body is known: everything but the body.
fn overhead_for(display: &str) -> usize {
    "===== ".len() + display.len() + NOTE_RESERVE_BYTES + 2
}

#[derive(Debug, PartialEq, Eq)]
struct RenderedBody {
    body: String,
    note: Option<String>,
}

/// Render one window of a file into at most `cap` bytes.
///
/// Every line carries its number. It costs about 2.3 tokens a line, +30% over a corpus of source
/// files, which is why it was conditional at first - on only for a window the caller asked for.
/// Measured against that: the model noticed ("the read display omitted line numbers!") and bought
/// them back with a corpus-wide `rg -n` sweep in two runs of three, at 9,675-12,598 tokens plus a
/// round trip, against ~14,300 to number every line outright. Roughly a wash on tokens, and the
/// numbers also let a citation be written from the read itself. Both agents this tool was
/// measured against number unconditionally.
fn render_body(
    contents: &str,
    offset: Option<usize>,
    limit: Option<usize>,
    numbered: bool,
    cap: usize,
) -> RenderedBody {
    let start = offset.unwrap_or(1).max(1) - 1;
    let limit = limit.unwrap_or(DEFAULT_LINE_LIMIT);
    let total_lines = contents.lines().count();

    let mut body = String::new();
    let mut kept = 0usize;
    let mut clipped = 0usize;
    let mut hit_cap = false;

    for line in contents.lines().skip(start).take(limit) {
        let mut rendered = String::new();
        if numbered {
            rendered.push_str(&format!("{}: ", start + kept + 1));
        }
        if line.len() > MAX_LINE_BYTES {
            rendered.push_str(clip_to_bytes(line, MAX_LINE_BYTES));
            rendered.push_str(" ... (line truncated)");
            clipped += 1;
        } else {
            rendered.push_str(line);
        }
        rendered.push('\n');

        if body.len() + rendered.len() > cap {
            // The first line alone can exceed a small cap. Emit what fits rather than an empty
            // section, so the model still learns the shape of the file.
            if body.is_empty() {
                const MARKER: &str = " ... (line truncated)\n";
                if cap > MARKER.len() {
                    body.push_str(clip_to_bytes(&rendered, cap - MARKER.len()));
                    body.push_str(MARKER);
                    kept += 1;
                    clipped += 1;
                }
            }
            hit_cap = true;
            break;
        }
        body.push_str(&rendered);
        kept += 1;
    }

    let shown_to = start + kept;
    let mut notes = Vec::new();
    if start >= total_lines && total_lines > 0 {
        notes.push(format!(
            "offset {} is past the end; {total_lines} lines",
            start + 1
        ));
    } else {
        if start > 0 {
            notes.push(format!("from line {}", start + 1));
        }
        if shown_to < total_lines {
            notes.push(format!(
                "showing {kept} of {total_lines} lines; continue with offset {}",
                shown_to + 1
            ));
        }
    }
    if clipped > 0 {
        notes.push(format!("{clipped} long line(s) clipped"));
    }
    if hit_cap && shown_to >= total_lines {
        notes.push("output budget reached".to_string());
    }

    let note = (!notes.is_empty()).then(|| {
        let joined = notes.join(", ");
        clip_to_bytes(&joined, NOTE_RESERVE_BYTES).to_string()
    });
    RenderedBody { body, note }
}

fn trailer(unread: &[String]) -> String {
    if unread.is_empty() {
        return String::new();
    }
    let shown: Vec<&str> = unread
        .iter()
        .take(TRAILER_MAX_NAMES)
        .map(|name| clip_to_bytes(name, TRAILER_NAME_MAX_BYTES))
        .collect();
    let hidden = unread.len().saturating_sub(shown.len());
    let names = if hidden > 0 {
        format!("{}, and {hidden} more", shown.join(", "))
    } else {
        shown.join(", ")
    };
    format!(
        "\n[read stopped after the output budget was reached. {} file(s) not read: {names}. \
         Ask for them in another call.]\n",
        unread.len()
    )
}

/// Concatenate the sections, carrying a running total so the bound holds whatever the allocator
/// decided. A section that would break it is named in the trailer instead of being emitted, which
/// is what makes the bound structural rather than a property of the arithmetic above.
fn assemble(
    sections: Vec<Section>,
    mut unread: Vec<String>,
    budget: usize,
    squeezed: Option<usize>,
) -> String {
    let ceiling = budget.saturating_sub(TRAILER_RESERVE_BYTES);
    let mut out = String::new();
    let mut stopped = false;
    for section in sections {
        if stopped || out.len() + section.len() > ceiling {
            stopped = true;
            unread.push(section.display);
            continue;
        }
        match &section.note {
            Some(note) => out.push_str(&format!("===== {} ({note})\n", section.display)),
            None => out.push_str(&format!("===== {}\n", section.display)),
        }
        out.push_str(&section.body);
        out.push('\n');
    }
    out.push_str(&trailer(&unread));
    // Per-file notes say a file was cut, but not that the *call* was too big, so a model that
    // asked for 2600 lines across 13 files has nothing to correct on its next try. One recorded
    // run followed such a call with `limit: 20` and `limit: 10` probes, each a whole round trip.
    if let Some(files) = squeezed {
        out.push_str(&format!(
            "
[the budget, not the files, decided what fit: {files} file(s) shared {budget}              bytes. Ask for fewer files per call, or a smaller `limit` per file.]
"
        ));
    }
    debug_assert!(
        out.len() <= budget,
        "read output must stay under its own budget, got {}",
        out.len()
    );
    out
}

// ---------------------------------------------------------------------------
// Budget
// ---------------------------------------------------------------------------

/// How many bytes this call may emit, given the model whose history will carry the result.
///
/// `record_items_with_metadata` truncates a `function_call_output` middle-out at
/// `truncation_policy * 1.2`, which is exactly the cut this tool exists to make unnecessary - a
/// middle-out cut lands inside a file and the model is not told which one. Filling four fifths of
/// that leaves room for the harness's own framing and for `byte_budget` being an estimate.
fn call_budget(policy: TruncationPolicy, auto_compact_tokens: Option<i64>) -> usize {
    let harness_cut =
        (policy * 1.2).byte_budget() * BUDGET_HEADROOM_NUMERATOR / BUDGET_HEADROOM_DENOMINATOR;
    // A model with no known context window has no compaction bound either, so the harness cut is
    // all there is to go on.
    let window_share = auto_compact_tokens
        .and_then(|tokens| usize::try_from(tokens).ok())
        .map_or(usize::MAX, |tokens| {
            TruncationPolicy::Tokens(tokens).byte_budget() / WINDOW_SHARE_DENOMINATOR
        });
    harness_cut
        .min(window_share)
        .clamp(MIN_CALL_BUDGET_BYTES, MAX_CALL_BUDGET_BYTES)
}

/// Split `content` bytes max-min fair over `demands`.
///
/// First-come-first-served is what made a twenty-file batch useless: three large files ate the
/// whole budget and seventeen came back named but unread. Water-filling gives every file an equal
/// share, and a file that wants less than its share hands the remainder back to the ones that want
/// more. Terminates in at most one round per entry and uses no floating point, so it is stable
/// across platforms and easy to pin in a test.
fn allocate(demands: &[usize], content: usize) -> Vec<usize> {
    let mut caps = vec![0usize; demands.len()];
    let mut unsatisfied: Vec<usize> = (0..demands.len()).collect();
    let mut remaining = content;

    while !unsatisfied.is_empty() {
        let share = remaining / unsatisfied.len();
        let (satisfied, rest): (Vec<usize>, Vec<usize>) = unsatisfied
            .iter()
            .copied()
            .partition(|index| demands[*index] <= share);
        if satisfied.is_empty() {
            for index in unsatisfied {
                caps[index] = share.min(PER_FILE_BUDGET_BYTES);
            }
            break;
        }
        for index in satisfied {
            caps[index] = demands[index];
            remaining -= demands[index];
        }
        unsatisfied = rest;
    }
    caps
}

// ---------------------------------------------------------------------------
// Binary and size guards
// ---------------------------------------------------------------------------

fn is_control_byte(byte: u8) -> bool {
    byte < 0x09 || byte == 0x0B || byte == 0x0C || (0x0E..=0x1F).contains(&byte) || byte == 0x7F
}

fn has_binary_extension(display: &str) -> bool {
    let name = display
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(display)
        .to_ascii_lowercase();
    match name.rsplit_once('.') {
        Some((stem, extension)) => !stem.is_empty() && BINARY_EXTENSIONS.contains(&extension),
        None => false,
    }
}

/// Decide from the head of the file, the way OpenCode does: an extension that is never text, a NUL
/// byte, or a control-byte share above 30%.
fn sniff_binary(display: &str, bytes: &[u8]) -> bool {
    if has_binary_extension(display) {
        return true;
    }
    let sample = &bytes[..bytes.len().min(BINARY_SAMPLE_BYTES)];
    if sample.contains(&0) {
        return true;
    }
    if sample.is_empty() {
        return false;
    }
    let control = sample.iter().filter(|byte| is_control_byte(**byte)).count();
    control * 100 > sample.len() * BINARY_CONTROL_PERCENT
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// What is left to do for one entry once the phase before it has run.
enum Plan {
    /// Already decided; the string is the whole body.
    Note(String),
    File {
        uri: PathUri,
        offset: Option<usize>,
        limit: Option<usize>,
        numbered: bool,
        demand: usize,
        cap: usize,
    },
    Directory {
        uri: PathUri,
    },
    /// Named in the trailer rather than read.
    Dropped,
}

struct Item {
    display: String,
    plan: Plan,
}

impl ReadHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            step_context,
            turn,
            payload,
            ..
        } = invocation;
        // `auto_compact_token_limit()` is the method, not the field: it folds in the config
        // override and clamps to 90% of the context window, and it is what the compaction check
        // itself reads (`session/context_window.rs`), so both are bound to one number.
        let budget = call_budget(
            turn.model_info().truncation_policy.into(),
            turn.model_info().auto_compact_token_limit(),
        );

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "read handler received unsupported payload".to_string(),
                ));
            }
        };

        let NormalizedArgs {
            targets,
            environment_id,
            mut overflow,
        } = normalize_args(&arguments)?;
        // One file per call. Reading the first path and dropping the rest would hide the shape of
        // the tool; naming the parallel form turns a rejected call into one round trip rather than
        // two, and the model already sends several calls in a response for searches.
        if targets.len() + overflow.len() > 1 {
            return Err(FunctionCallError::RespondToModel(
                "`read` takes one `filePath`. To read several files, send several `read` calls \
                 in one response - they run together and each answers on its own."
                    .to_string(),
            ));
        }

        let Some(turn_environment) =
            resolve_tool_environment(&step_context.environments, environment_id.as_deref())?
        else {
            return Err(FunctionCallError::RespondToModel(
                "read is unavailable in this session".to_string(),
            ));
        };
        let sandbox = turn_environment.sandbox_context(/*additional_permissions*/ None);
        let sandbox = Some(&sandbox);
        let filesystem = turn_environment.environment.get_filesystem();
        let filesystem = filesystem.as_ref();

        // Phase 1: resolve and dedup. Containment stays the sandbox's job, so `join` is used rather
        // than `join_descendant`: the model routinely feeds back absolute paths it got from `rg`.
        let mut items: Vec<Item> = Vec::with_capacity(targets.len());
        let mut seen: Vec<(String, Option<usize>, Option<usize>)> = Vec::new();
        for target in targets {
            let uri = match turn_environment.cwd().join(&target.raw) {
                Ok(uri) => uri,
                Err(error) => {
                    items.push(Item {
                        display: target.raw,
                        plan: Plan::Note(format!("unable to resolve path: {error}")),
                    });
                    continue;
                }
            };
            let display = uri.inferred_native_path_string();
            let key = (display.clone(), target.offset, target.limit);
            if seen.contains(&key) {
                items.push(Item {
                    display,
                    plan: Plan::Note("already shown above".to_string()),
                });
                continue;
            }
            seen.push(key);
            items.push(Item {
                display,
                plan: Plan::File {
                    uri,
                    offset: target.offset,
                    limit: target.limit,
                    numbered: true,
                    demand: 0,
                    cap: 0,
                },
            });
        }

        // Phase 2: metadata, concurrently, order preserved by `buffered`. The stream is driven over
        // owned values rather than borrowed items so the closure is not higher-ranked over a
        // lifetime it cannot name; `agents_md.rs` drives its probes the same way.
        let probes: Vec<Option<PathUri>> = items
            .iter()
            .map(|item| match &item.plan {
                Plan::File { uri, .. } => Some(uri.clone()),
                _ => None,
            })
            .collect();
        let metadata: Vec<Option<io::Result<FileMetadata>>> = futures::stream::iter(probes)
            .map(|uri| async move {
                match uri {
                    Some(uri) => Some(
                        filesystem
                            .get_metadata(&uri, GetMetadataOptions::default(), sandbox)
                            .await,
                    ),
                    None => None,
                }
            })
            .buffered(MAX_CONCURRENT_METADATA)
            .collect()
            .await;

        for (item, metadata) in items.iter_mut().zip(metadata) {
            let Some(metadata) = metadata else { continue };
            let Plan::File {
                uri, limit, demand, ..
            } = &mut item.plan
            else {
                continue;
            };
            match metadata {
                Err(error) => item.plan = Plan::Note(format!("unable to read: {error}")),
                Ok(metadata) if metadata.is_directory => {
                    item.plan = Plan::Directory { uri: uri.clone() }
                }
                Ok(metadata) if !metadata.is_file => {
                    item.plan = Plan::Note("not a file".to_string())
                }
                // Refuse from the metadata: reading a 500 MB file into memory to discover it is too
                // large is the expensive way to learn it.
                Ok(metadata) if metadata.size > MAX_FILE_BYTES => {
                    item.plan = Plan::Note(format!(
                        "file is {} bytes; too large to read. Use `rg` to find the lines you \
                         need, then read a window.",
                        metadata.size
                    ))
                }
                Ok(metadata) if metadata.size == 0 => {
                    item.plan = Plan::Note("empty file".to_string())
                }
                Ok(metadata) => {
                    let size = usize::try_from(metadata.size).unwrap_or(usize::MAX);
                    // The budget bounds the *output*, and every line leaves with its number in
                    // front of it, so a demand taken from the file's own size under-provisions by
                    // exactly the numbering and truncates every file a little. A two-line file of
                    // 11 bytes leaves as 17.
                    let numbered = size
                        .saturating_add(size / NUMBERING_OVERHEAD_DIVISOR)
                        .saturating_add(NUMBERING_OVERHEAD_FLOOR_BYTES);
                    let mut want = numbered.min(PER_FILE_BUDGET_BYTES);
                    if let Some(lines) = limit {
                        // Rough planning estimate: asking for fewer lines should not reserve the
                        // whole file. Reserving too little only clips early, and the note says so.
                        want = want.min(lines.saturating_mul(100));
                    }
                    *demand = want;
                }
            }
        }

        // Phase 3: allocate. Overhead is charged for every section, the short ones included, so a
        // batch that is entirely errors is bounded by the same arithmetic as a batch of files.
        let fixed: usize = items
            .iter()
            .map(|item| overhead_for(&item.display))
            .sum::<usize>()
            + TRAILER_RESERVE_BYTES;
        let mut readable: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| matches!(item.plan, Plan::File { .. }))
            .map(|(index, _)| index)
            .collect();
        let mut unread: Vec<String> = Vec::new();

        // Drop from the tail until what is left can be given a usable slice each.
        while !readable.is_empty() && fixed + readable.len() * MIN_FILE_BUDGET_BYTES > budget {
            let Some(index) = readable.pop() else { break };
            unread.push(items[index].display.clone());
            items[index].plan = Plan::Dropped;
        }
        let content = budget.saturating_sub(fixed);
        let demands: Vec<usize> = readable
            .iter()
            .map(|index| match items[*index].plan {
                Plan::File { demand, .. } => demand,
                _ => 0,
            })
            .collect();
        let caps = allocate(&demands, content);
        // The call was oversubscribed when the share, not the file's own size, is what cut it.
        let squeezed = caps
            .iter()
            .zip(&demands)
            .any(|(cap, demand)| cap < demand)
            .then_some(readable.len());
        for (index, allocated) in readable.iter().zip(caps) {
            if let Plan::File { cap, .. } = &mut items[*index].plan {
                *cap = allocated;
            }
        }

        // Phase 4: read, concurrently, order preserved.
        let to_read: Vec<Option<PathUri>> = items
            .iter()
            .map(|item| match &item.plan {
                Plan::File { uri, .. } => Some(uri.clone()),
                _ => None,
            })
            .collect();
        let contents: Vec<Option<io::Result<Vec<u8>>>> = futures::stream::iter(to_read)
            .map(|uri| async move {
                match uri {
                    Some(uri) => Some(
                        filesystem
                            .read_file(&uri, ReadFileOptions::default(), sandbox)
                            .await,
                    ),
                    None => None,
                }
            })
            .buffered(MAX_CONCURRENT_READS)
            .collect()
            .await;

        // Phase 5: directory listings, concurrently.
        let to_list: Vec<Option<PathUri>> = items
            .iter()
            .map(|item| match &item.plan {
                Plan::Directory { uri } => Some(uri.clone()),
                _ => None,
            })
            .collect();
        let listings: Vec<Option<io::Result<Vec<ReadDirectoryEntry>>>> =
            futures::stream::iter(to_list)
                .map(|uri| async move {
                    match uri {
                        Some(uri) => Some(filesystem.read_directory(&uri, sandbox).await),
                        None => None,
                    }
                })
                .buffered(MAX_CONCURRENT_METADATA)
                .collect()
                .await;

        // The three vectors are indexed by entry, and `zip` would silently drop the tail if one of
        // them were short, losing a file the model asked for without saying so.
        debug_assert_eq!(items.len(), contents.len(), "one read result per entry");
        debug_assert_eq!(items.len(), listings.len(), "one listing slot per entry");

        let mut sections: Vec<Section> = Vec::with_capacity(items.len());
        for ((item, bytes), listing) in items.into_iter().zip(contents).zip(listings) {
            let Item { display, plan } = item;
            match plan {
                Plan::Note(reason) => sections.push(Section::note_only(display, &reason)),
                Plan::Dropped => {}
                Plan::Directory { .. } => sections.push(match listing {
                    Some(Ok(entries)) => render_directory(display, entries),
                    Some(Err(error)) => {
                        Section::note_only(display, &format!("unable to read: {error}"))
                    }
                    None => Section::note_only(display, "unable to read"),
                }),
                Plan::File {
                    offset,
                    limit,
                    numbered,
                    cap,
                    ..
                } => sections.push(match bytes {
                    Some(Ok(bytes)) => {
                        if sniff_binary(&display, &bytes) {
                            Section::note_only(display, "cannot read binary file")
                        } else {
                            // Lossy rather than strict: one latin-1 comment in an otherwise fine
                            // source file should not cost the model the whole file. The sniff
                            // above, not the decoder, is the binary gate.
                            let text = String::from_utf8_lossy(&bytes);
                            let rendered = render_body(&text, offset, limit, numbered, cap);
                            Section {
                                display,
                                note: rendered.note,
                                body: rendered.body,
                            }
                        }
                    }
                    Some(Err(error)) => {
                        Section::note_only(display, &format!("unable to read: {error}"))
                    }
                    None => Section::note_only(display, "unable to read"),
                }),
            }
        }

        unread.append(&mut overflow);
        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            assemble(sections, unread, budget, squeezed),
            Some(true),
        )))
    }
}

/// A directory answered with `not a file` cost a round trip on `ls` before the model could read
/// anything; listing it here costs one extra filesystem call instead.
fn render_directory(display: String, mut entries: Vec<ReadDirectoryEntry>) -> Section {
    entries.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    let total = entries.len();
    let mut body = String::new();
    let mut shown = 0usize;
    for entry in entries.iter().take(MAX_DIRECTORY_ENTRIES) {
        let line = if entry.is_directory {
            format!("{}/\n", entry.file_name)
        } else {
            format!("{}\n", entry.file_name)
        };
        if body.len() + line.len() > DIRECTORY_BUDGET_BYTES {
            break;
        }
        body.push_str(&line);
        shown += 1;
    }
    if shown < total {
        body.push_str(&format!("(+{} more)\n", total - shown));
    }
    Section {
        display,
        note: Some(format!("directory; {total} entries")),
        body,
    }
}

impl ToolExecutor<ToolInvocation> for ReadHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(READ_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_read_tool(self.options)
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(self.handle_call(invocation))
    }
}

impl CoreToolRuntime for ReadHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(arguments: &str) -> Vec<ReadTarget> {
        normalize_args(arguments)
            .expect("arguments must normalize")
            .targets
    }

    fn target(raw: &str, offset: Option<usize>, limit: Option<usize>) -> ReadTarget {
        ReadTarget {
            raw: raw.to_string(),
            offset,
            limit,
        }
    }

    #[test]
    fn a_bare_string_entry_is_a_whole_file() {
        assert_eq!(
            targets(r#"{"paths":["a.rs","b.rs"]}"#),
            vec![target("a.rs", None, None), target("b.rs", None, None)]
        );
    }

    #[test]
    fn an_object_entry_carries_its_own_window() {
        assert_eq!(
            targets(r#"{"paths":["a.rs",{"path":"b.rs","offset":40,"limit":80}]}"#),
            vec![
                target("a.rs", None, None),
                target("b.rs", Some(40), Some(80))
            ]
        );
    }

    #[test]
    fn a_top_level_limit_reaches_every_entry_that_set_none() {
        // The call that this tool was rewritten for: the model believed it was asking for 500 lines
        // of each file and silently received all of every file. It now means what the model reads
        // it to mean. Dividing it instead was measured and reverted - see `normalize_args`.
        assert_eq!(
            targets(r#"{"limit":500,"paths":["a.rs","b.rs","c.rs"]}"#),
            vec![
                target("a.rs", None, Some(500)),
                target("b.rs", None, Some(500)),
                target("c.rs", None, Some(500)),
            ]
        );
    }

    #[test]
    fn a_top_level_offset_still_reaches_every_entry() {
        // A position applies to each file, exactly like the limit beside it.
        let shared = targets(r#"{"offset":40,"paths":["a.rs","b.rs"]}"#);
        assert!(
            shared.iter().all(|target| target.offset == Some(40)),
            "{shared:?}"
        );
    }

    #[test]
    fn an_entry_window_wins_over_the_top_level_one() {
        assert_eq!(
            targets(r#"{"limit":500,"paths":[{"path":"a.rs","limit":20},"b.rs"]}"#),
            vec![
                target("a.rs", None, Some(20)),
                target("b.rs", None, Some(500))
            ]
        );
    }

    #[test]
    fn alternate_spellings_of_the_list_and_path_keys_are_accepted() {
        for arguments in [
            r#"{"files":["a.rs"]}"#,
            r#"{"file_paths":["a.rs"]}"#,
            r#"{"filePaths":["a.rs"]}"#,
            r#"{"paths":"a.rs"}"#,
            r#"{"path":"a.rs"}"#,
            r#"{"file_path":"a.rs"}"#,
            r#"{"filePath":"a.rs"}"#,
            r#"{"paths":[{"file":"a.rs"}]}"#,
        ] {
            assert_eq!(
                targets(arguments),
                vec![target("a.rs", None, None)],
                "{arguments}"
            );
        }
    }

    #[test]
    fn inclusive_end_line_becomes_a_limit() {
        assert_eq!(
            targets(r#"{"paths":[{"path":"a.rs","start_line":40,"end_line":80}]}"#),
            vec![target("a.rs", Some(40), Some(41))]
        );
    }

    #[test]
    fn an_orphan_range_folds_into_the_entry_before_it() {
        assert_eq!(
            targets(r#"{"paths":["a.rs",{"start_line":10,"end_line":19}]}"#),
            vec![target("a.rs", Some(10), Some(10))]
        );
    }

    #[test]
    fn quoted_numbers_and_unknown_keys_are_tolerated() {
        assert_eq!(
            targets(r#"{"paths":[{"path":"a.rs","offset":"120","limit":"5","why":"because"}]}"#),
            vec![target("a.rs", Some(120), Some(5))]
        );
    }

    #[test]
    fn double_encoded_arguments_are_parsed_once_more() {
        assert_eq!(
            targets(r#""{\"paths\":[\"a.rs\"]}""#),
            vec![target("a.rs", None, None)]
        );
    }

    #[test]
    fn entries_past_the_cap_are_named_rather_than_dropped() {
        let paths: Vec<String> = (0..MAX_ENTRIES + 3)
            .map(|index| format!("\"f{index}.rs\""))
            .collect();
        let normalized = normalize_args(&format!("{{\"paths\":[{}]}}", paths.join(",")))
            .expect("arguments must normalize");
        assert_eq!(normalized.targets.len(), MAX_ENTRIES);
        assert_eq!(normalized.overflow.len(), 3);
        assert_eq!(normalized.overflow[0], format!("f{MAX_ENTRIES}.rs"));
    }

    #[test]
    fn an_unusable_call_is_answered_with_an_example() {
        for arguments in [r#"{"paths":[]}"#, r#"{}"#, "not json"] {
            let error = normalize_args(arguments).expect_err("must be rejected");
            let FunctionCallError::RespondToModel(message) = error else {
                panic!("read must answer the model, not fail the turn");
            };
            assert!(message.contains("\"offset\": 40"), "{message}");
        }
    }

    #[test]
    fn a_whole_file_read_is_numbered_from_one() {
        let rendered = render_body(
            "a
b
c
",
            None,
            None,
            true,
            MAX_CALL_BUDGET_BYTES,
        );
        assert_eq!(
            rendered.body,
            "1: a
2: b
3: c
"
        );
        assert!(rendered.note.is_none());
    }

    #[test]
    fn a_window_is_numbered_from_its_own_offset() {
        let contents = (1..=10)
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let rendered = render_body(&contents, Some(3), Some(2), true, MAX_CALL_BUDGET_BYTES);
        assert_eq!(rendered.body, "3: 3\n4: 4\n");
        let note = rendered.note.expect("a partial read must say so");
        assert!(note.contains("from line 3"), "{note}");
        assert!(note.contains("continue with offset 5"), "{note}");
    }

    #[test]
    fn an_offset_past_the_end_says_so_instead_of_returning_nothing() {
        let rendered = render_body("a\nb\n", Some(40), None, true, MAX_CALL_BUDGET_BYTES);
        assert!(rendered.body.is_empty());
        let note = rendered.note.expect("a read past the end must say so");
        assert!(note.contains("past the end"), "{note}");
        assert!(note.contains("2 lines"), "{note}");
    }

    #[test]
    fn long_lines_are_clipped_rather_than_dropped() {
        let contents = format!("{}\nshort\n", "x".repeat(MAX_LINE_BYTES + 50));
        let rendered = render_body(&contents, None, None, false, MAX_CALL_BUDGET_BYTES);
        assert!(rendered.body.contains("... (line truncated)"));
        assert!(rendered.body.contains("short"));
        let note = rendered.note.expect("clipping must be reported");
        assert!(note.contains("1 long line(s) clipped"), "{note}");
    }

    #[test]
    fn a_multibyte_line_is_clipped_on_a_char_boundary() {
        let contents = format!("{}\n", "ü".repeat(MAX_LINE_BYTES));
        let rendered = render_body(&contents, None, None, false, MAX_CALL_BUDGET_BYTES);
        assert!(rendered.body.contains("... (line truncated)"));
        assert!(rendered.body.is_char_boundary(rendered.body.len()));
    }

    #[test]
    fn a_body_never_exceeds_its_cap() {
        let contents = (1..=4_000)
            .map(|line| format!("line {line} of a long file"))
            .collect::<Vec<_>>()
            .join("\n");
        for cap in [0usize, 8, 64, 1_000, 20_000] {
            let rendered = render_body(&contents, None, None, false, cap);
            assert!(
                rendered.body.len() <= cap,
                "cap {cap} exceeded by {}",
                rendered.body.len()
            );
        }
    }

    #[test]
    fn carriage_returns_do_not_reach_the_model() {
        let rendered = render_body("a\r\nb\r\n", None, None, false, MAX_CALL_BUDGET_BYTES);
        assert_eq!(rendered.body, "a\nb\n");
    }

    #[test]
    fn the_budget_follows_the_model_that_will_carry_the_output() {
        // glm-5.3-flash. The harness cut allows 96_000, but the 80k window allows only a sixth of
        // its 320_000 bytes, and the window is the binding limit: a call that fills 96_000 is 29%
        // of the window, and the measured runs that did that compacted twice.
        assert_eq!(
            call_budget(TruncationPolicy::Tokens(25_000), Some(80_000)),
            53_333
        );
        // A model with the common 10k policy is cut at 48_000 by the harness, which binds first.
        assert_eq!(
            call_budget(TruncationPolicy::Tokens(10_000), Some(80_000)),
            38_400
        );
        // No known context window means no compaction bound to respect.
        assert_eq!(call_budget(TruncationPolicy::Tokens(25_000), None), 96_000);
        assert_eq!(
            call_budget(TruncationPolicy::Tokens(100_000), Some(4_000_000)),
            MAX_CALL_BUDGET_BYTES
        );
        assert_eq!(
            call_budget(TruncationPolicy::Bytes(1_000), Some(80_000)),
            MIN_CALL_BUDGET_BYTES
        );
        // A tiny window binds even when the policy is generous.
        assert_eq!(
            call_budget(TruncationPolicy::Tokens(25_000), Some(15_000)),
            10_000
        );
    }

    #[test]
    fn a_squeezed_call_is_told_that_the_budget_was_the_limit() {
        let section = Section::note_only("a.rs".to_string(), "x");
        let squeezed = assemble(vec![section], Vec::new(), MAX_CALL_BUDGET_BYTES, Some(13));
        assert!(squeezed.contains("the budget, not the files"), "{squeezed}");
        assert!(squeezed.contains("13 file(s)"), "{squeezed}");

        let section = Section::note_only("a.rs".to_string(), "x");
        let roomy = assemble(vec![section], Vec::new(), MAX_CALL_BUDGET_BYTES, None);
        assert!(
            !roomy.contains("the budget, not the files"),
            "a call that fit must not be told it did not: {roomy}"
        );
    }

    #[test]
    fn a_small_file_hands_its_surplus_to_a_large_one() {
        assert_eq!(
            allocate(&[1_000, 1_000, 500_000], 30_000),
            vec![1_000, 1_000, 28_000]
        );
    }

    #[test]
    fn equal_demands_are_split_equally() {
        let demands = vec![PER_FILE_BUDGET_BYTES; 40];
        let caps = allocate(&demands, 80_000);
        assert!(caps.iter().all(|cap| *cap == 2_000), "{caps:?}");
    }

    #[test]
    fn an_allocation_never_exceeds_the_per_file_ceiling_or_the_content_budget() {
        let demands = vec![10, 500_000, 3_000, 900_000, 40];
        let caps = allocate(&demands, 90_000);
        assert!(
            caps.iter().all(|cap| *cap <= PER_FILE_BUDGET_BYTES),
            "{caps:?}"
        );
        assert!(caps.iter().sum::<usize>() <= 90_000, "{caps:?}");
    }

    #[test]
    fn assembling_stays_under_the_call_budget_on_adversarial_input() {
        let sections: Vec<Section> = (0..MAX_ENTRIES)
            .map(|index| Section {
                display: format!("{}/{index}.rs", "d".repeat(1_000)),
                note: Some("x".repeat(NOTE_RESERVE_BYTES)),
                body: "y".repeat(PER_FILE_BUDGET_BYTES),
            })
            .collect();
        let out = assemble(sections, Vec::new(), MAX_CALL_BUDGET_BYTES, None);
        assert!(out.len() <= MAX_CALL_BUDGET_BYTES, "got {}", out.len());
        assert!(
            out.contains("[read stopped"),
            "sections that did not fit must be named"
        );
    }

    #[test]
    fn the_trailer_names_at_most_ten_files_and_counts_the_rest() {
        let unread: Vec<String> = (0..25).map(|index| format!("f{index}.rs")).collect();
        let text = trailer(&unread);
        assert!(text.contains("25 file(s) not read"), "{text}");
        assert!(text.contains("and 15 more"), "{text}");
        assert!(text.len() <= TRAILER_RESERVE_BYTES, "got {}", text.len());
    }

    #[test]
    fn binary_is_detected_from_the_extension_a_nul_byte_or_control_bytes() {
        assert!(sniff_binary("C:\\p\\logo.png", b"not really a png"));
        assert!(sniff_binary("a.rs", b"fn main() {}\0"));
        assert!(sniff_binary("a.rs", &[0x01u8; 64]));
        assert!(!sniff_binary(
            "a.rs",
            "fn main() { println!(\"hi\"); }".as_bytes()
        ));
        assert!(!sniff_binary("notes.md", "büyük harfli metin".as_bytes()));
        assert!(!sniff_binary("a.rs", b""));
        // A dotfile has no extension, whatever comes after the dot.
        assert!(!sniff_binary(".gitignore", b"target/"));
    }
}
