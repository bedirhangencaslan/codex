use super::*;
use pretty_assertions::assert_eq;

fn text(text: &str) -> FunctionCallOutputContentItem {
    FunctionCallOutputContentItem::InputText {
        text: text.to_string(),
    }
}

fn texts(items: &[FunctionCallOutputContentItem]) -> Vec<&str> {
    items
        .iter()
        .map(|item| match item {
            FunctionCallOutputContentItem::InputText { text } => text.as_str(),
            other => panic!("expected text, got {other:?}"),
        })
        .collect()
}

/// A successful `cargo build` whose output is almost all progress lines.
fn cargo_build_output() -> String {
    let mut lines: Vec<String> = (0..300)
        .map(|index| format!("   Compiling crate_{index} v0.1.0"))
        .collect();
    lines.push("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.10s".into());
    lines.join("\n")
}

fn cargo_build_record(output: &str) -> NestedExecRecord {
    NestedExecRecord {
        arguments: r#"{"cmd":"cargo build"}"#.to_string(),
        exit_code: Some(0),
        process_id: None,
        output: output.to_string(),
    }
}

#[test]
fn fair_shares_leave_a_fitting_result_alone() {
    assert_eq!(fair_shares(&[10, 20, 30], 100), vec![None, None, None]);
}

#[test]
fn fair_shares_keep_small_items_whole_and_split_the_rest_evenly() {
    // 100 bytes for three items and two separators: the 10-byte item fits its third, and the
    // two large ones split what it left.
    assert_eq!(
        fair_shares(&[500, 10, 400], 100),
        vec![Some(44), None, Some(44)]
    );
}

#[test]
fn a_script_answer_survives_next_to_a_dump_that_is_cut() {
    let answer = "3 of 88 components have no timeout";
    let dump = "x".repeat(10_000);
    let items = vec![text(answer), text(&dump)];

    let shaped = shape_text_items(&items, &[], TruncationPolicy::Tokens(500), None)
        .expect("text-only output is shaped");

    let shaped = texts(&shaped);
    assert_eq!(shaped[0], answer);
    assert!(shaped[1].starts_with("Warning: truncated output"));
    assert!(shaped[1].contains("tokens truncated"));
    assert!(shaped[1].len() < dump.len());
}

#[test]
fn a_single_item_is_cut_exactly_as_upstream_cuts_it() {
    // One item with no spill target is the case upstream already handled; the marker, its unit
    // and the frame must come out byte for byte the same.
    let items = vec![text(&"0123456789".repeat(40))];

    let shaped = shape_text_items(&items, &[], TruncationPolicy::Tokens(20), None)
        .expect("text-only output is shaped");

    let (upstream, _) = codex_utils_output_truncation::formatted_truncate_text_content_items_with_policy(
        &items,
        TruncationPolicy::Tokens(20),
    );
    assert_eq!(shaped, upstream);
    assert!(texts(&shaped)[0].contains("tokens truncated"));
}

#[test]
fn a_verbatim_echo_of_a_nested_build_is_condensed_like_the_direct_path() {
    let output = cargo_build_output();
    let items = vec![text(&output)];

    let shaped = shape_text_items(
        &items,
        &[cargo_build_record(&output)],
        TruncationPolicy::Tokens(100_000),
        None,
    )
    .expect("text-only output is shaped");

    let shaped = texts(&shaped);
    assert!(
        shaped[0].starts_with("Harness trimmed"),
        "expected the lossy notice, got: {}",
        &shaped[0][..shaped[0].len().min(200)]
    );
    assert!(shaped[0].contains("Finished `dev` profile"));
    assert!(shaped[0].len() < output.len() / 4);
}

#[test]
fn text_the_script_computed_is_never_condensed() {
    // One character different from the command's output: the script's own text, not an echo.
    let output = cargo_build_output();
    let computed = format!("{output}!");
    let items = vec![text(&computed)];

    let shaped = shape_text_items(
        &items,
        &[cargo_build_record(&output)],
        TruncationPolicy::Tokens(100_000),
        None,
    )
    .expect("text-only output is shaped");

    assert_eq!(texts(&shaped), vec![computed.as_str()]);
}

#[test]
fn a_cut_item_is_saved_whole_and_named() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = SpillTarget {
        dir: dir.path().to_path_buf(),
        stem: "call-7".to_string(),
    };
    let dump = (0..5_000)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let items = vec![text("done"), text(&dump)];

    let shaped = shape_text_items(&items, &[], TruncationPolicy::Tokens(2_000), Some(&target))
        .expect("text-only output is shaped");

    let shaped = texts(&shaped);
    assert_eq!(shaped[0], "done");
    let path = dir.path().join("code_mode_call-7_1.txt");
    assert!(
        shaped[1].starts_with(&format!("Full output saved to: {}", path.display())),
        "got: {}",
        &shaped[1][..shaped[1].len().min(200)]
    );
    assert_eq!(std::fs::read_to_string(&path).expect("spill file"), dump);
    assert!(shaped.iter().map(|item| item.len()).sum::<usize>() <= 2_000 * 4 + 200);
}

#[test]
fn a_condensed_echo_that_fits_still_saves_what_it_dropped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = SpillTarget {
        dir: dir.path().to_path_buf(),
        stem: "call-8".to_string(),
    };
    let output = cargo_build_output();

    let shaped = shape_text_items(
        &[text(&output)],
        &[cargo_build_record(&output)],
        TruncationPolicy::Tokens(100_000),
        Some(&target),
    )
    .expect("text-only output is shaped");

    let path = dir.path().join("code_mode_call-8_0.txt");
    assert!(texts(&shaped)[0].starts_with("Full output saved to: "));
    assert_eq!(std::fs::read_to_string(&path).expect("spill file"), output);
}

#[test]
fn media_output_keeps_upstreams_handling() {
    let items = vec![
        text("caption"),
        FunctionCallOutputContentItem::InputAudio {
            audio_url: "data:audio/wav;base64,AAAA".to_string(),
        },
    ];
    assert_eq!(
        shape_text_items(&items, &[], TruncationPolicy::Tokens(10), None),
        None
    );
}

#[test]
fn the_ledger_forgets_a_finished_cell_and_keeps_a_yielded_one() {
    let ledger = NestedExecLedger::default();
    ledger.record("cell-1", cargo_build_record("a"));

    assert_eq!(ledger.records("cell-1", /*finished*/ false).len(), 1);
    assert_eq!(ledger.records("cell-1", /*finished*/ true).len(), 1);
    assert_eq!(ledger.records("cell-1", /*finished*/ true).len(), 0);
}

#[test]
fn the_ledger_keeps_only_the_most_recent_records() {
    let ledger = NestedExecLedger::default();
    for index in 0..(MAX_RECORDS_PER_CELL + 5) {
        ledger.record("cell-1", cargo_build_record(&format!("run {index}")));
    }

    let records = ledger.records("cell-1", /*finished*/ true);
    assert_eq!(records.len(), MAX_RECORDS_PER_CELL);
    assert_eq!(records[0].output, "run 5");
}
