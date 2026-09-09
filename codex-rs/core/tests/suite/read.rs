#![allow(clippy::unwrap_used)]

use anyhow::Result;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use serde_json::Value;

/// Seed the workspace, let the model issue one `read` call, and hand back what the tool put in the
/// `function_call_output`. That text is the whole contract with the model, so every assertion below
/// is made against it rather than against an internal.
async fn read_output(files: &[(&str, String)], arguments: Value) -> Result<String> {
    let server = start_mock_server().await;
    let mocks = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call("call-1", "read", &arguments.to_string()),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-1", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;

    let mut builder = test_codex();
    let test = builder.build_with_auto_env(&server).await?;
    for (relative, contents) in files {
        let path = test.workspace_path(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)?;
    }

    test.submit_turn("read the files").await?;
    Ok(mocks
        .function_call_output_text("call-1")
        .expect("read must produce a function_call_output"))
}

fn numbered_lines(count: usize) -> String {
    (1..=count)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_window_is_numbered_from_its_own_offset_and_says_how_to_continue() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let output = read_output(
        &[("windowed.txt", numbered_lines(200))],
        serde_json::json!({ "filePath": "windowed.txt", "offset": 5, "limit": 2 }),
    )
    .await?;

    // A window is numbered from its own offset, so a `file:line` citation can be written from the
    // read itself rather than from a second pass with `rg -n`, and the read can be resumed.
    assert!(output.contains("5: line 5
6: line 6
"), "{output}");
    assert!(output.contains("from line 5"), "{output}");
    assert!(output.contains("continue with offset 7"), "{output}");
    Ok(())
}

/// The batched shape is refused rather than half-served: reading the first path and dropping the
/// rest would leave the model believing it had them.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn several_paths_are_refused_with_the_parallel_form() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let output = read_output(
        &[
            ("a.txt", "alpha
".to_string()),
            ("b.txt", "beta
".to_string()),
        ],
        serde_json::json!({ "paths": ["a.txt", "b.txt"] }),
    )
    .await?;

    assert!(output.contains("one `filePath`"), "{output}");
    assert!(output.contains("in one response"), "{output}");
    assert!(!output.contains("1: alpha"), "no file may be served: {output}");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_binary_file_is_refused_without_leaking_its_bytes() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let mut contents = String::from("SECRETMARKER");
    contents.push('\0');
    contents.push_str("more bytes");

    let output = read_output(
        &[("blob.bin", contents)],
        serde_json::json!({ "filePath": "blob.bin" }),
    )
    .await?;

    assert!(output.contains("cannot read binary file"), "{output}");
    assert!(
        !output.contains("SECRETMARKER"),
        "a refused file must not have its bytes emitted anyway: {output}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn one_large_file_stays_inside_the_calls_budget() -> Result<()> {
    skip_if_no_network!(Ok(()));

    // A single file can still exceed what one tool output may carry, and the cut has to announce
    // itself: the model needs the offset to continue from, not a silently short answer.
    let output = read_output(
        &[("big.txt", numbered_lines(30_000))],
        serde_json::json!({ "filePath": "big.txt" }),
    )
    .await?;

    assert!(output.contains("1: line 1
"), "{output}");
    assert!(
        output.contains("continue with offset"),
        "a cut read must say where to resume: {output}"
    );
    assert!(
        output.len() < 60_000,
        "the call budget must bound one file too, got {} bytes",
        output.len()
    );
    Ok(())
}
