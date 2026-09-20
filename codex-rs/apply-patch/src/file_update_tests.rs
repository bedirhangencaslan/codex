use super::*;
use crate::Hunk;
use crate::apply_patch;
use crate::parse_patch;
use codex_exec_server::LOCAL_FS;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;
use std::fs;
use tempfile::tempdir;

fn wrap_patch(body: &str) -> String {
    format!("*** Begin Patch\n{body}\n*** End Patch")
}

#[tokio::test]
async fn test_unified_diff() {
    // Start with a file containing four lines.
    let dir = tempdir().unwrap();
    let path = dir.path().join("multi.txt");
    fs::write(&path, "foo\nbar\nbaz\nqux\n").unwrap();
    let patch = wrap_patch(&format!(
        r#"*** Update File: {}
@@
 foo
-bar
+BAR
@@
 baz
-qux
+QUX"#,
        path.display()
    ));
    let patch = parse_patch(&patch).unwrap();

    let update_file_chunks = match patch.hunks.as_slice() {
        [Hunk::UpdateFile { chunks, .. }] => chunks,
        _ => panic!("Expected a single UpdateFile hunk"),
    };
    let path_uri = PathUri::from_host_native_path(&path).expect("absolute test path");
    let diff = unified_diff_from_chunks(
        &path_uri,
        update_file_chunks,
        LOCAL_FS.as_ref(),
        /*sandbox*/ None,
    )
    .await
    .unwrap();
    let expected_diff = r#"@@ -1,4 +1,4 @@
 foo
-bar
+BAR
 baz
-qux
+QUX
"#;
    let expected = ApplyPatchFileUpdate {
        unified_diff: expected_diff.to_string(),
        original_content: "foo\nbar\nbaz\nqux\n".to_string(),
        content: "foo\nBAR\nbaz\nQUX\n".to_string(),
    };
    assert_eq!(expected, diff);
}

#[tokio::test]
async fn test_unified_diff_first_line_replacement() {
    // Replace the very first line of the file.
    let dir = tempdir().unwrap();
    let path = dir.path().join("first.txt");
    fs::write(&path, "foo\nbar\nbaz\n").unwrap();

    let patch = wrap_patch(&format!(
        r#"*** Update File: {}
@@
-foo
+FOO
 bar
"#,
        path.display()
    ));

    let patch = parse_patch(&patch).unwrap();
    let chunks = match patch.hunks.as_slice() {
        [Hunk::UpdateFile { chunks, .. }] => chunks,
        _ => panic!("Expected a single UpdateFile hunk"),
    };

    let resolved_path = PathUri::from_host_native_path(&path).expect("absolute test path");
    let diff = unified_diff_from_chunks(
        &resolved_path,
        chunks,
        LOCAL_FS.as_ref(),
        /*sandbox*/ None,
    )
    .await
    .unwrap();
    let expected_diff = r#"@@ -1,2 +1,2 @@
-foo
+FOO
 bar
"#;
    let expected = ApplyPatchFileUpdate {
        unified_diff: expected_diff.to_string(),
        original_content: "foo\nbar\nbaz\n".to_string(),
        content: "FOO\nbar\nbaz\n".to_string(),
    };
    assert_eq!(expected, diff);
}

#[tokio::test]
async fn test_unified_diff_last_line_replacement() {
    // Replace the very last line of the file.
    let dir = tempdir().unwrap();
    let path = dir.path().join("last.txt");
    fs::write(&path, "foo\nbar\nbaz\n").unwrap();

    let patch = wrap_patch(&format!(
        r#"*** Update File: {}
@@
 foo
 bar
-baz
+BAZ
"#,
        path.display()
    ));

    let patch = parse_patch(&patch).unwrap();
    let chunks = match patch.hunks.as_slice() {
        [Hunk::UpdateFile { chunks, .. }] => chunks,
        _ => panic!("Expected a single UpdateFile hunk"),
    };

    let resolved_path = PathUri::from_host_native_path(&path).expect("absolute test path");
    let diff = unified_diff_from_chunks(
        &resolved_path,
        chunks,
        LOCAL_FS.as_ref(),
        /*sandbox*/ None,
    )
    .await
    .unwrap();
    let expected_diff = r#"@@ -2,2 +2,2 @@
 bar
-baz
+BAZ
"#;
    let expected = ApplyPatchFileUpdate {
        unified_diff: expected_diff.to_string(),
        original_content: "foo\nbar\nbaz\n".to_string(),
        content: "foo\nbar\nBAZ\n".to_string(),
    };
    assert_eq!(expected, diff);
}

#[tokio::test]
async fn test_unified_diff_insert_at_eof() {
    // Insert a new line at end-of-file.
    let dir = tempdir().unwrap();
    let path = dir.path().join("insert.txt");
    fs::write(&path, "foo\nbar\nbaz\n").unwrap();

    let patch = wrap_patch(&format!(
        r#"*** Update File: {}
@@
+quux
*** End of File
"#,
        path.display()
    ));

    let patch = parse_patch(&patch).unwrap();
    let chunks = match patch.hunks.as_slice() {
        [Hunk::UpdateFile { chunks, .. }] => chunks,
        _ => panic!("Expected a single UpdateFile hunk"),
    };

    let path_uri = PathUri::from_host_native_path(&path).expect("absolute test path");
    let diff =
        unified_diff_from_chunks(&path_uri, chunks, LOCAL_FS.as_ref(), /*sandbox*/ None)
            .await
            .unwrap();
    let expected_diff = r#"@@ -3 +3,2 @@
 baz
+quux
"#;
    let expected = ApplyPatchFileUpdate {
        unified_diff: expected_diff.to_string(),
        original_content: "foo\nbar\nbaz\n".to_string(),
        content: "foo\nbar\nbaz\nquux\n".to_string(),
    };
    assert_eq!(expected, diff);
}

#[tokio::test]
async fn test_unified_diff_interleaved_changes() {
    // Original file with six lines.
    let dir = tempdir().unwrap();
    let path = dir.path().join("interleaved.txt");
    fs::write(&path, "a\nb\nc\nd\ne\nf\n").unwrap();

    // Patch replaces two separate lines and appends a new one at EOF using
    // three distinct chunks.
    let patch_body = format!(
        r#"*** Update File: {}
@@
 a
-b
+B
@@
 d
-e
+E
@@
 f
+g
*** End of File"#,
        path.display()
    );
    let patch = wrap_patch(&patch_body);

    // Extract chunks then build the unified diff.
    let parsed = parse_patch(&patch).unwrap();
    let chunks = match parsed.hunks.as_slice() {
        [Hunk::UpdateFile { chunks, .. }] => chunks,
        _ => panic!("Expected a single UpdateFile hunk"),
    };

    let path_uri = PathUri::from_host_native_path(&path).expect("absolute test path");
    let diff =
        unified_diff_from_chunks(&path_uri, chunks, LOCAL_FS.as_ref(), /*sandbox*/ None)
            .await
            .unwrap();

    let expected_diff = r#"@@ -1,6 +1,7 @@
 a
-b
+B
 c
 d
-e
+E
 f
+g
"#;

    let expected = ApplyPatchFileUpdate {
        unified_diff: expected_diff.to_string(),
        original_content: "a\nb\nc\nd\ne\nf\n".to_string(),
        content: "a\nB\nc\nd\nE\nf\ng\n".to_string(),
    };

    assert_eq!(expected, diff);

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    apply_patch(
        &patch,
        &PathUri::from_host_native_path(dir.path()).expect("absolute test path"),
        &mut stdout,
        &mut stderr,
        LOCAL_FS.as_ref(),
        /*sandbox*/ None,
    )
    .await
    .unwrap();
    let contents = fs::read_to_string(path).unwrap();
    assert_eq!(
        contents,
        r#"a
B
c
d
E
f
g
"#
    );
}

fn to_lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

#[test]
fn stale_hunk_quotes_current_text_instead_of_the_senders_own_lines() {
    // The file moved on; the hunk still carries the sender's older memory of it.
    let original = to_lines("def parse(sql):\n    tokens = _lex(sql)\n    return _build(tokens, strict=True)");
    let pattern = to_lines("def parse(sql):\n    tokens = _lex(sql)\n    return _build(tokens)");

    let message = missing_lines_error("demo.py", &original, &pattern, 0).to_string();

    // The one thing the sender cannot know: what is there now, and at which line.
    assert!(
        message.contains("1: def parse(sql):"),
        "expected numbered current text: {message}"
    );
    assert!(
        message.contains("3:     return _build(tokens, strict=True)"),
        "expected the file's current third line: {message}"
    );
    // The stale line the sender just wrote is not echoed back at it.
    assert!(
        !message.contains("return _build(tokens)\n"),
        "hunk body should not be echoed: {message}"
    );
    assert!(message.contains("(2 more lines in this hunk)"), "{message}");
}

#[test]
fn quoted_context_stays_within_its_line_and_byte_bounds() {
    let original: Vec<String> = (1..=500).map(|n| format!("line {n}")).collect();
    // Matches at the top, then diverges, so a quote block is produced.
    let mut pattern = original[..50].to_vec();
    pattern.push("a line that is not in the file".to_string());

    let message = missing_lines_error("big.rs", &original, &pattern, 0).to_string();

    let quoted = message.lines().filter(|l| l.starts_with("1: ") || l.contains(": line ")).count();
    assert!(
        quoted <= FAILURE_CONTEXT_LINES,
        "quoted {quoted} lines, cap is {FAILURE_CONTEXT_LINES}: {message}"
    );
    assert!(
        message.len() < FAILURE_CONTEXT_BYTES + 400,
        "message grew to {} bytes: {message}",
        message.len()
    );
}

#[test]
fn large_stale_hunk_costs_less_than_echoing_it_back() {
    // The design is a swap, not an addition: what used to be spent echoing the hunk is spent on
    // the file's current text instead, under a fixed cap.
    let original: Vec<String> = (1..=500)
        .map(|n| format!("    let value_{n} = compute_something_reasonably_long({n});"))
        .collect();
    let mut pattern = original[..50].to_vec();
    pattern.push("    let missing = not_in_the_file();".to_string());

    let previous_behaviour = format!(
        "Failed to find expected lines in big.rs:\n{}",
        pattern.join("\n")
    );
    let message = missing_lines_error("big.rs", &original, &pattern, 0).to_string();

    assert!(
        message.len() < previous_behaviour.len(),
        "new message is {} bytes against the old {}",
        message.len(),
        previous_behaviour.len()
    );
}

#[test]
fn hunk_matching_only_before_the_cursor_is_reported_as_out_of_order() {
    let original = to_lines("alpha\nbeta\ngamma");
    let pattern = to_lines("alpha");

    // An earlier hunk already advanced past line 1.
    let message = missing_lines_error("ordered.txt", &original, &pattern, 2).to_string();

    assert!(
        message.contains("These lines appear at line 1"),
        "expected an ordering diagnostic: {message}"
    );
}

#[test]
fn hunk_absent_from_the_file_says_so_with_the_current_length() {
    let original = to_lines("alpha\nbeta");
    let pattern = to_lines("nowhere to be found");

    let message = missing_lines_error("absent.txt", &original, &pattern, 0).to_string();

    assert!(
        message.contains("No line of this hunk appears in the file, which now has 2 lines."),
        "{message}"
    );
}

#[test]
fn best_partial_match_prefers_the_densest_overlap() {
    let lines = to_lines("a\nb\nc\na\nb\nz");
    let pattern = to_lines("a\nb\nz");

    // Offset 0 matches two of three lines; offset 3 matches all three.
    assert_eq!(best_partial_match(&lines, &pattern, 0), Some((3, 3)));
    // Nothing lines up once the cursor is past the dense match.
    assert_eq!(best_partial_match(&lines, &pattern, 4), None);
    // A start at or beyond the end of the file has nowhere to look.
    assert_eq!(best_partial_match(&lines, &pattern, 6), None);

    // A partial overlap is still an anchor when no full match exists anywhere.
    let truncated = to_lines("a\nb\nc");
    assert_eq!(best_partial_match(&truncated, &pattern, 0), Some((0, 2)));
}

#[test]
fn best_partial_match_ignores_indentation_drift() {
    let lines = to_lines("        return value");
    let pattern = to_lines("    return value");

    assert_eq!(best_partial_match(&lines, &pattern, 0), Some((0, 1)));
}
