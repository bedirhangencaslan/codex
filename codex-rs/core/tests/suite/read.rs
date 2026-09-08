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
async fn a_batch_mixes_whole_files_with_windows_and_numbers_every_line() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let output = read_output(
        &[
            ("plain.txt", "alpha\nbeta\n".to_string()),
            ("windowed.txt", numbered_lines(200)),
        ],
        serde_json::json!({
            "paths": [
                "plain.txt",
                { "path": "windowed.txt", "offset": 5, "limit": 2 },
            ]
        }),
    )
    .await?;

    let plain_at = output.find("plain.txt").expect("plain.txt must appear");
    let windowed_at = output
        .find("windowed.txt")
        .expect("windowed.txt must appear");
    assert!(
        plain_at < windowed_at,
        "sections must keep the order they were asked for: {output}"
    );

    // Every read is numbered, so a `file:line` citation can be written from the read itself
    // rather than from a second pass with `rg -n`.
    assert!(output.contains("1: alpha\n2: beta\n"), "{output}");

    // A window is numbered from its own offset, so it can be cited and resumed.
    assert!(output.contains("5: line 5\n6: line 6\n"), "{output}");
    assert!(output.contains("from line 5"), "{output}");
    assert!(output.contains("continue with offset 7"), "{output}");
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
        serde_json::json!({ "paths": ["blob.bin"] }),
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
async fn a_top_level_limit_windows_every_file_and_the_call_stays_inside_its_budget() -> Result<()> {
    skip_if_no_network!(Ok(()));

    // The call that motivated this rewrite: a top-level `limit` alongside a dozen paths. It used to
    // be dropped in silence and return twelve whole files.
    let files: Vec<(String, String)> = (0..12)
        .map(|index| (format!("big{index}.txt"), numbered_lines(3_000)))
        .collect();
    let borrowed: Vec<(&str, String)> = files
        .iter()
        .map(|(name, contents)| (name.as_str(), contents.clone()))
        .collect();
    let paths: Vec<String> = files.iter().map(|(name, _)| name.clone()).collect();

    let output = read_output(
        &borrowed,
        serde_json::json!({ "limit": 500, "paths": paths }),
    )
    .await?;

    let missing: Vec<&str> = files
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !output.contains(*name))
        .collect();
    assert!(
        missing.is_empty(),
        "every file asked for must have a section; missing {missing:?} of {} bytes",
        output.len()
    );
    assert_eq!(
        output.matches("showing").count(),
        12,
        "every file must report that it was windowed: {output}"
    );
    assert!(
        !output.contains("line 3000"),
        "a windowed read must not return the whole file"
    );
    // The point of the tool deciding where to stop: the harness never has to cut the output
    // middle-out, which would land inside a file without telling the model which one.
    assert!(
        !output.contains("tokens truncated"),
        "read must stay inside the budget its model allows, got {} bytes",
        output.len()
    );
    Ok(())
}
