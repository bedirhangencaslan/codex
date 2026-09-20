use codex_exec_server::ExecutorFileSystem;
use codex_exec_server::FileSystemSandboxContext;
use codex_exec_server::ReadFileOptions;
use codex_utils_path_uri::PathUri;
use similar::TextDiff;

use crate::ApplyPatchError;
use crate::ApplyPatchFileUpdateMode;
use crate::IoError;
use crate::UpdateFileChunk;
use crate::seek_sequence;
use crate::text_file::Replacement;
use crate::text_file::SourceFile;

#[cfg(test)]
#[path = "file_update_tests.rs"]
mod tests;

pub(crate) struct AppliedPatch {
    pub(crate) original_contents: String,
    pub(crate) new_contents: String,
}

/// Return *only* the new file contents (joined into a single `String`) after
/// applying the chunks to the file at `path`.
pub(crate) async fn derive_new_contents_from_chunks(
    path: &PathUri,
    chunks: &[UpdateFileChunk],
    update_file_mode: ApplyPatchFileUpdateMode,
    fs: &dyn ExecutorFileSystem,
    follow_symlinks: bool,
    sandbox: Option<&FileSystemSandboxContext>,
) -> std::result::Result<AppliedPatch, ApplyPatchError> {
    let original_contents = fs
        .read_file_text(path, ReadFileOptions { follow_symlinks }, sandbox)
        .await
        .map_err(|err| {
            ApplyPatchError::IoError(IoError {
                context: format!(
                    "Failed to read file to update {}",
                    path.inferred_native_path_string()
                ),
                source: err,
            })
        })?;

    let path_text = path.inferred_native_path_string();
    let new_contents = match update_file_mode {
        ApplyPatchFileUpdateMode::NormalizeToLf => {
            let mut original_lines = original_contents
                .split('\n')
                .map(String::from)
                .collect::<Vec<_>>();

            // Drop the trailing empty element that results from the final newline so
            // that line counts match the behaviour of standard `diff`.
            if original_lines.last().is_some_and(String::is_empty) {
                original_lines.pop();
            }

            let replacements =
                compute_replacements(&original_lines, &path_text, chunks, update_file_mode)?;
            let mut new_lines = apply_replacements(original_lines, &replacements);
            if !new_lines.last().is_some_and(String::is_empty) {
                new_lines.push(String::new());
            }
            new_lines.join("\n")
        }
        ApplyPatchFileUpdateMode::PreserveLineEndings => {
            let mut source_file = SourceFile::parse(&original_contents);
            let original_lines = source_file.line_texts();
            let replacements =
                compute_replacements(&original_lines, &path_text, chunks, update_file_mode)?;
            source_file.apply_replacements(&replacements);
            source_file.into_contents()
        }
    };
    Ok(AppliedPatch {
        original_contents,
        new_contents,
    })
}

/// Compute a list of replacements needed to transform `original_lines` into the
/// new lines, given the patch `chunks`. Each replacement is returned as
/// `(start_index, old_len, new_lines)`.
fn compute_replacements(
    original_lines: &[String],
    path: &str,
    chunks: &[UpdateFileChunk],
    update_file_mode: ApplyPatchFileUpdateMode,
) -> std::result::Result<Vec<Replacement>, ApplyPatchError> {
    let mut replacements: Vec<Replacement> = Vec::new();
    let mut line_index: usize = 0;

    for chunk in chunks {
        // If a chunk has a `change_context`, we use seek_sequence to find it, then
        // adjust our `line_index` to continue from there.
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(idx) = seek_sequence::seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                /*eof*/ false,
                update_file_mode,
            ) {
                line_index = idx + 1;
            } else {
                return Err(ApplyPatchError::ComputeReplacements(format!(
                    "Failed to find context '{ctx_line}' in {path}"
                )));
            }
        }

        if chunk.old_lines.is_empty() {
            // Preserve the legacy split representation's handling of a final
            // empty line. `SourceFile` only exposes real source lines, so its
            // insertion point is always after the final line.
            let insertion_idx = match update_file_mode {
                ApplyPatchFileUpdateMode::NormalizeToLf => {
                    if original_lines.last().is_some_and(String::is_empty) {
                        original_lines.len() - 1
                    } else {
                        original_lines.len()
                    }
                }
                ApplyPatchFileUpdateMode::PreserveLineEndings => original_lines.len(),
            };
            replacements.push((insertion_idx, 0, chunk.new_lines.clone()));
            continue;
        }

        // Otherwise, try to match the existing lines in the file with the old lines
        // from the chunk. If found, schedule that region for replacement.
        // Attempt to locate the `old_lines` verbatim within the file.  In many
        // real‑world diffs the last element of `old_lines` is an *empty* string
        // representing the terminating newline of the region being replaced.
        // This sentinel is not present in `original_lines` because `SourceFile`
        // stores the terminator on the preceding line rather than as an extra
        // trailing element. If a direct search fails and the pattern ends with
        // an empty string, retry without that final element so modifications
        // touching the end‑of‑file can be located reliably.

        let mut pattern: &[String] = &chunk.old_lines;
        let mut found = seek_sequence::seek_sequence(
            original_lines,
            pattern,
            line_index,
            chunk.is_end_of_file,
            update_file_mode,
        );

        let mut new_slice: &[String] = &chunk.new_lines;

        if found.is_none() && pattern.last().is_some_and(String::is_empty) {
            // Retry without the trailing empty line which represents the final
            // newline in the file.
            pattern = &pattern[..pattern.len() - 1];
            if new_slice.last().is_some_and(String::is_empty) {
                new_slice = &new_slice[..new_slice.len() - 1];
            }

            found = seek_sequence::seek_sequence(
                original_lines,
                pattern,
                line_index,
                chunk.is_end_of_file,
                update_file_mode,
            );
        }

        if let Some(start_idx) = found {
            match update_file_mode {
                ApplyPatchFileUpdateMode::NormalizeToLf => {
                    replacements.push((start_idx, pattern.len(), new_slice.to_vec()));
                }
                ApplyPatchFileUpdateMode::PreserveLineEndings => {
                    // Context lines occur in both sides of a patch chunk. Keep those
                    // original lines in place so their exact contents and terminators
                    // survive, especially when the file has mixed line endings.
                    let mut old_start = 0;
                    let mut new_start = 0;
                    for &(old_context, new_context) in &chunk.context_line_indices {
                        // A trailing empty context line can be removed from `pattern`
                        // and `new_slice` above when it represents the final newline.
                        if old_context >= pattern.len() || new_context >= new_slice.len() {
                            break;
                        }
                        if old_start != old_context || new_start != new_context {
                            replacements.push((
                                start_idx + old_start,
                                old_context - old_start,
                                new_slice[new_start..new_context].to_vec(),
                            ));
                        }
                        old_start = old_context + 1;
                        new_start = new_context + 1;
                    }
                    if old_start != pattern.len() || new_start != new_slice.len() {
                        replacements.push((
                            start_idx + old_start,
                            pattern.len() - old_start,
                            new_slice[new_start..].to_vec(),
                        ));
                    }
                }
            }
            line_index = start_idx + pattern.len();
        } else {
            return Err(missing_lines_error(
                path,
                original_lines,
                pattern,
                line_index,
            ));
        }
    }

    replacements.sort_by_key(|(index, _, _)| *index);

    Ok(replacements)
}

/// How much of the file to quote back when a hunk fails to apply.
///
/// This message used to echo `chunk.old_lines` — the sender's own patch text — straight back at
/// them, which tells them nothing they did not just write. What they cannot know is the file's
/// current text where the hunk was meant to land, and its current line numbers, which shift under
/// every hunk applied before it. These bounds keep that quote from dominating the result when a
/// hunk fails inside a long function body.
const FAILURE_CONTEXT_LINES: usize = 20;
const FAILURE_CONTEXT_BYTES: usize = 800;

/// Cap on how many pattern lines are scored per candidate offset, so that a pathologically long
/// hunk cannot make this error path quadratic over a large file.
const FAILURE_PROBE_LINES: usize = 64;

/// Best-effort anchor for a hunk that did not match: the offset at or after `start` where the
/// greatest number of `pattern` lines line up, comparing trimmed text so that a pure
/// indentation drift still anchors. Returns the offset and how many lines matched there, or
/// `None` when nothing lines up at all.
fn best_partial_match(lines: &[String], pattern: &[String], start: usize) -> Option<(usize, usize)> {
    let probe = pattern.len().min(FAILURE_PROBE_LINES);
    if probe == 0 || start >= lines.len() {
        return None;
    }

    let mut best_offset = 0usize;
    let mut best_score = 0usize;
    for offset in start..lines.len() {
        let mut score = 0usize;
        for (index, expected) in pattern.iter().take(probe).enumerate() {
            match lines.get(offset + index) {
                Some(actual) if actual.trim() == expected.trim() => score += 1,
                _ => {}
            }
        }
        if score > best_score {
            best_score = score;
            best_offset = offset;
        }
    }

    if best_score == 0 {
        None
    } else {
        Some((best_offset, best_score))
    }
}

/// Quote the file as it currently stands, starting at `anchor`, with 1-based line numbers and
/// bounded by `FAILURE_CONTEXT_LINES` / `FAILURE_CONTEXT_BYTES`.
fn quote_current_lines(lines: &[String], anchor: usize) -> Vec<String> {
    let mut quoted = Vec::new();
    let mut budget = FAILURE_CONTEXT_BYTES;
    for (offset, text) in lines
        .iter()
        .enumerate()
        .skip(anchor)
        .take(FAILURE_CONTEXT_LINES)
    {
        let rendered = format!("{}: {text}", offset + 1);
        if rendered.len() > budget {
            quoted.push("...".to_string());
            break;
        }
        budget -= rendered.len();
        quoted.push(rendered);
    }
    quoted
}

/// Build the failure for a hunk whose `old_lines` could not be located.
///
/// Keeps only the hunk's first line — enough to identify *which* hunk failed, and all that is
/// needed when a patch carries several — and spends the rest of the message on the one thing the
/// sender cannot recover without re-reading the file.
fn missing_lines_error(
    path: &str,
    original_lines: &[String],
    pattern: &[String],
    line_index: usize,
) -> ApplyPatchError {
    let mut message = vec![format!("Failed to find expected lines in {path}:")];
    message.push(pattern.first().cloned().unwrap_or_default());
    if pattern.len() > 1 {
        message.push(format!("({} more lines in this hunk)", pattern.len() - 1));
    }

    if let Some((anchor, score)) = best_partial_match(original_lines, pattern, line_index) {
        message.push(format!(
            "Closest match is at line {} ({score} of {} lines match). The file now reads:",
            anchor + 1,
            pattern.len(),
        ));
        message.extend(quote_current_lines(original_lines, anchor));
    } else if let Some((anchor, _)) = best_partial_match(original_lines, pattern, 0) {
        // Searching resumes after the previous hunk, so a match that only exists before
        // `line_index` means the hunks are not in file order rather than that the text is gone.
        message.push(format!(
            "These lines appear at line {}, before the point an earlier hunk in this patch \
             already advanced past. Hunks must be ordered as they appear in the file.",
            anchor + 1,
        ));
    } else {
        message.push(format!(
            "No line of this hunk appears in the file, which now has {} lines.",
            original_lines.len(),
        ));
    }

    ApplyPatchError::ComputeReplacements(message.join("\n"))
}

/// Apply the `(start_index, old_len, new_lines)` replacements to `original_lines`,
/// returning the modified file contents as a vector of lines.
fn apply_replacements(mut lines: Vec<String>, replacements: &[Replacement]) -> Vec<String> {
    // We must apply replacements in descending order so that earlier replacements
    // don't shift the positions of later ones.
    for (start_idx, old_len, new_segment) in replacements.iter().rev() {
        let start_idx = *start_idx;
        let old_len = *old_len;

        // Remove old lines.
        for _ in 0..old_len {
            if start_idx < lines.len() {
                lines.remove(start_idx);
            }
        }

        // Insert new lines.
        for (offset, new_line) in new_segment.iter().enumerate() {
            lines.insert(start_idx + offset, new_line.clone());
        }
    }

    lines
}

/// Intended result of a file update for apply_patch.
#[derive(Debug, Eq, PartialEq)]
pub struct ApplyPatchFileUpdate {
    pub(crate) unified_diff: String,
    pub(crate) original_content: String,
    pub(crate) content: String,
}

pub async fn unified_diff_from_chunks(
    path: &PathUri,
    chunks: &[UpdateFileChunk],
    fs: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
) -> std::result::Result<ApplyPatchFileUpdate, ApplyPatchError> {
    unified_diff_from_chunks_with_mode(
        path,
        chunks,
        ApplyPatchFileUpdateMode::default(),
        fs,
        sandbox,
    )
    .await
}

pub(crate) async fn unified_diff_from_chunks_with_mode(
    path: &PathUri,
    chunks: &[UpdateFileChunk],
    update_file_mode: ApplyPatchFileUpdateMode,
    fs: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
) -> std::result::Result<ApplyPatchFileUpdate, ApplyPatchError> {
    unified_diff_from_chunks_with_context_and_mode(
        path,
        chunks,
        /*context*/ 1,
        update_file_mode,
        fs,
        sandbox,
    )
    .await
}

pub async fn unified_diff_from_chunks_with_context(
    path: &PathUri,
    chunks: &[UpdateFileChunk],
    context: usize,
    fs: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
) -> std::result::Result<ApplyPatchFileUpdate, ApplyPatchError> {
    unified_diff_from_chunks_with_context_and_mode(
        path,
        chunks,
        context,
        ApplyPatchFileUpdateMode::default(),
        fs,
        sandbox,
    )
    .await
}

async fn unified_diff_from_chunks_with_context_and_mode(
    path: &PathUri,
    chunks: &[UpdateFileChunk],
    context: usize,
    update_file_mode: ApplyPatchFileUpdateMode,
    fs: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
) -> std::result::Result<ApplyPatchFileUpdate, ApplyPatchError> {
    let AppliedPatch {
        original_contents,
        new_contents,
    } = derive_new_contents_from_chunks(
        path,
        chunks,
        update_file_mode,
        fs,
        /*follow_symlinks*/ true,
        sandbox,
    )
    .await?;
    let text_diff = TextDiff::from_lines(&original_contents, &new_contents);
    let unified_diff = text_diff.unified_diff().context_radius(context).to_string();
    Ok(ApplyPatchFileUpdate {
        unified_diff,
        original_content: original_contents,
        content: new_contents,
    })
}
