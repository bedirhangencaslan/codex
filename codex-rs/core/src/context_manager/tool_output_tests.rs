use super::*;

/// Output long enough to be worth shrinking, with a recognizable first and last line.
///
/// The filler is deliberately not a build receipt. It used to be `   Compiling crate-N`, which
/// the progress filter now removes outright - leaving two lines, nothing for `shrink_text` to do,
/// and these tests asserting a mechanism that no longer had to run. The subject here is head/tail
/// shrinking, so the filler has to be content that only shrinking can remove.
fn noisy_output() -> String {
    let mut lines = vec!["first".to_string()];
    for index in 0..200 {
        lines.push(format!("note: detail line {index}"));
    }
    lines.push("last".to_string());
    lines.join("\n")
}

/// A recursive listing whose project files are buried under its dependency tree.
fn listing(project_lines: usize, artifact_lines: usize) -> String {
    let mut lines = Vec::new();
    for index in 0..project_lines {
        lines.push(format!("C:\\work\\app\\src\\page-{index}.tsx"));
    }
    for index in 0..artifact_lines {
        lines.push(format!(
            "C:\\work\\app\\node_modules\\dep-{index}\\index.js"
        ));
    }
    lines.join("\n")
}

/// Runs the live condenser over one finished call's arguments and output.
fn condense_pair(arguments: &str, text: &str, success: Option<bool>) -> String {
    let exit_code = match success {
        Some(true) => Some(0),
        Some(false) => Some(1),
        None => None,
    };
    condense_exec_output(arguments, exit_code, /*process_id*/ None, text)
        .map(|(condensed, _)| condensed)
        .unwrap_or_else(|| text.to_string())
}

fn condense_command(command: &str) -> String {
    let arguments = serde_json::json!({ "command": command }).to_string();
    condense_pair(&arguments, &noisy_output(), Some(true))
}

fn report_for(arguments: &str, text: &str) -> CondenseReport {
    condense_exec_output(arguments, Some(0), /*process_id*/ None, text)
        .map(|(_, report)| report)
        .unwrap_or_default()
}

fn report_for_failure(arguments: &str, text: &str) -> CondenseReport {
    condense_exec_output(arguments, Some(1), /*process_id*/ None, text)
        .map(|(_, report)| report)
        .unwrap_or_default()
}

fn was_shrunk(text: &str) -> bool {
    text.contains("[system] ") && text.contains(" lines trimmed")
}

/// A build whose warnings sit in the middle, under a long receipt.
///
/// This is the shape the progress filter exists for: head/tail shrinking alone keeps five lines
/// of `Compiling` and the last thirty, and the warnings in between are what it drops.
fn build_with_buried_warnings(crates: usize) -> String {
    let mut lines = Vec::new();
    for index in 0..crates {
        lines.push(format!("   Compiling crate-{index} v0.1.{index}"));
    }
    for index in 0..6 {
        lines.push(format!(
            "warning: unused variable `value_{index}` in src/lib.rs"
        ));
        lines.push(format!("  --> src/lib.rs:{}:9", index + 40));
    }
    lines.push("    Finished `dev` profile in 12.3s".to_string());
    lines.join("\n")
}

fn cargo_arguments(command: &str) -> String {
    serde_json::json!({ "command": command }).to_string()
}

#[test]
fn a_build_receipt_goes_and_the_warnings_under_it_stay() {
    let text = build_with_buried_warnings(/*crates*/ 120);
    let arguments = cargo_arguments("cargo build");

    let condensed = condense_pair(&arguments, &text, Some(true));

    assert!(!condensed.contains("Compiling crate-"), "{condensed}");
    // Every warning survives, which head/tail shrinking on its own could not promise.
    for index in 0..6 {
        assert!(
            condensed.contains(&format!("unused variable `value_{index}`")),
            "warning {index} was dropped:\n{condensed}"
        );
    }
    // The one status line worth a line: it says the build reached the end.
    assert!(condensed.contains("Finished `dev` profile"), "{condensed}");
    assert!(condensed.len() < text.len());
}

#[test]
fn the_header_says_how_many_progress_lines_went() {
    let report = report_for(
        &cargo_arguments("cargo build"),
        &build_with_buried_warnings(/*crates*/ 120),
    );

    assert_eq!(report.progress_lines, 120);
    let summary = report.summary().expect("a lossy stage ran");
    assert!(summary.contains("120 progress lines"), "{summary}");
}

#[test]
fn a_passing_test_roll_call_goes_and_the_result_line_stays() {
    let mut lines = vec!["running 40 tests".to_string()];
    for index in 0..40 {
        lines.push(format!("test suite::case_{index} ... ok"));
    }
    lines.push("test suite::flaky ... FAILED".to_string());
    lines.push("test result: FAILED. 40 passed; 1 failed; 0 ignored".to_string());
    let text = lines.join("\n");

    let condensed = condense_pair(&cargo_arguments("cargo test"), &text, Some(true));

    assert!(!condensed.contains("... ok"), "{condensed}");
    assert!(!condensed.contains("running 40 tests"), "{condensed}");
    // A failing test and the counts are the answer, not the receipt.
    assert!(
        condensed.contains("test suite::flaky ... FAILED"),
        "{condensed}"
    );
    assert!(condensed.contains("test result: FAILED."), "{condensed}");
}

#[test]
fn a_programs_own_output_is_not_mistaken_for_a_build_receipt() {
    // Same verbs, column zero. Cargo's arrive indented; a test's `println!` does not, and this
    // stage drops what it matches rather than rewriting it.
    let mut lines = Vec::new();
    for index in 0..40 {
        lines.push(format!("Compiling shader pass {index}"));
        lines.push(format!("Checking mesh {index}"));
    }
    let text = lines.join("\n");

    let condensed = condense_pair(&cargo_arguments("cargo run"), &text, Some(true));

    assert!(condensed.contains("Compiling shader pass 0"), "{condensed}");
    assert!(condensed.contains("Checking mesh 39"), "{condensed}");
}

/// One `rustc` diagnostic, in the shape a real `cargo build` prints: the header at column zero
/// and every continuation line indented.
fn error_block(index: usize) -> String {
    format!(
        "error[E0308]: mismatched types\n \
         --> src/lib.rs:{index}:22\n  \
         |\n{index} |     let wrong: i32 = \"not a number\";\n  \
         |                ---   ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n"
    )
}

/// A failing build: a long receipt, `count` errors, and the tail rustc closes with.
fn failing_build(crates: usize, errors: usize) -> String {
    let mut out = String::new();
    for index in 0..crates {
        out.push_str(&format!("   Compiling crate-{index} v0.1.{index}\n"));
    }
    for index in 0..errors {
        out.push_str(&error_block(index));
        out.push('\n');
    }
    out.push_str("Some errors have detailed explanations: E0308.\n");
    out.push_str(&format!(
        "error: could not compile `probe` (lib) due to {errors} previous errors"
    ));
    out
}

#[test]
fn a_failing_build_loses_its_receipt_but_not_its_first_errors() {
    let text = failing_build(/*crates*/ 120, /*errors*/ 40);

    let condensed = condense_pair(&cargo_arguments("cargo build"), &text, Some(false));

    // The receipt is not the answer on a failure either - it is what stands between the model
    // and the first error.
    assert!(!condensed.contains("Compiling crate-"), "{condensed}");
    // Every error up to the cap survives whole, header and all four continuation lines.
    assert_eq!(condensed.matches("error[E0308]").count(), MAX_ERROR_BLOCKS);
    assert!(
        condensed.contains("expected `i32`, found `&str`"),
        "{condensed}"
    );
    // rustc's own closing count is never a block and never capped: it is how bad it is.
    assert!(
        condensed.contains("error: could not compile `probe` (lib) due to 40 previous errors"),
        "{condensed}"
    );
    assert!(
        condensed.contains("Some errors have detailed explanations"),
        "{condensed}"
    );
}

#[test]
fn the_header_counts_the_errors_it_did_not_carry() {
    let report = report_for_failure(
        &cargo_arguments("cargo build"),
        &failing_build(/*crates*/ 120, /*errors*/ 40),
    );

    assert_eq!(report.dropped_errors, 40 - MAX_ERROR_BLOCKS);
    let summary = report.summary().expect("a lossy stage ran");
    assert!(summary.contains("20 further errors"), "{summary}");
}

#[test]
fn a_diagnostic_is_carried_whole_or_not_at_all() {
    let text = failing_build(/*crates*/ 0, /*errors*/ 40);

    let condensed = condense_pair(&cargo_arguments("cargo build"), &text, Some(false));

    // A block cut in half is worse than one that is absent and counted, because the model cannot
    // tell which it is reading. So the continuation lines track the headers exactly.
    assert_eq!(
        condensed.matches("--> src/lib.rs:").count(),
        condensed.matches("error[E0308]").count()
    );
}

#[test]
fn a_failure_with_few_errors_is_left_exactly_as_it_came() {
    // Three errors and no receipt is all signal. Nothing is over a cap, nothing is progress, and
    // the whole thing is well under the notice's worth - so the model gets it verbatim.
    let text = failing_build(/*crates*/ 0, /*errors*/ 3);

    assert_eq!(
        condense_pair(&cargo_arguments("cargo build"), &text, Some(false)),
        text
    );
}

/// One pytest failure, in the shape a real run prints: an underscore header, the frame, and the
/// `_ _ _ _` sub-separator pytest puts *between* frames of the same traceback.
fn pytest_failure(name: &str) -> String {
    format!(
        "____________________________ {name} _____________________________\n\
         \n    def {name}():\n>       helper([\"not\", \"a\", \"dict\"])\n\n\
         test_probe.py:22: \n\
         _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _\n\
         \nvalue = ['not', 'a', 'dict']\n\n    def helper(value):\n\
         >       return value[\"missing\"]\n\
         E       TypeError: list indices must be integers or slices, not str\n\n\
         test_probe.py:6: TypeError\n"
    )
}

/// A failing pytest run: session header, the dot line, `count` failures, and the two closing
/// sections.
fn failing_pytest(count: usize) -> String {
    let mut out = String::from(
        "============================= test session starts ==============================\n",
    );
    out.push_str("platform win32 -- Python 3.13.15, pytest-9.1.1, pluggy-1.6.0\n");
    out.push_str("rootdir: C:\\work\\project\n");
    out.push_str("plugins: anyio-4.15.1, typeguard-4.6.0\n");
    out.push_str(&format!("collected {} items\n\n", count + 2));
    out.push_str(
        "test_probe.py ..FFF                                                      [100%]\n\n",
    );
    out.push_str(
        "================================== FAILURES ===================================\n",
    );
    for index in 0..count {
        out.push_str(&pytest_failure(&format!("test_case_{index}")));
    }
    out.push_str(
        "=========================== short test summary info ===========================\n",
    );
    for index in 0..count {
        out.push_str(&format!(
            "FAILED test_probe.py::test_case_{index} - TypeError: list indices must b...\n"
        ));
    }
    out.push_str(&format!(
        "========================= {count} failed, 2 passed in 0.17s ========================="
    ));
    out
}

#[test]
fn a_pytest_run_loses_its_session_header_and_its_dot_line() {
    let condensed = condense_pair(
        &cargo_arguments("python -m pytest"),
        &failing_pytest(/*count*/ 30),
        Some(false),
    );

    assert!(!condensed.contains("platform win32"), "{condensed}");
    assert!(!condensed.contains("rootdir:"), "{condensed}");
    assert!(!condensed.contains("plugins:"), "{condensed}");
    assert!(!condensed.contains("[100%]"), "{condensed}");
    // The denominator stays: it is what every count below it is a fraction of.
    assert!(condensed.contains("collected 32 items"), "{condensed}");
}

#[test]
fn a_pytest_traceback_is_not_cut_into_frames() {
    let condensed = condense_pair(
        &cargo_arguments("python -m pytest"),
        &failing_pytest(/*count*/ 30),
        Some(false),
    );

    // Three underscores is a failure header; `_ _ _ _` divides frames inside one. Matching the
    // latter would cap a single traceback to pieces, so the frames of every carried failure are
    // still there - and there are as many sub-separators as carried failures.
    let carried = condensed.matches("____ test_case_").count();
    assert_eq!(carried, MAX_ERROR_BLOCKS);
    // Counted as lines, not as substrings: one separator line contains the pattern several times
    // over, which is how this assertion first read 120 for twenty tracebacks.
    let separators = condensed
        .lines()
        .filter(|line| line.starts_with("_ _"))
        .count();
    assert_eq!(separators, carried);
    // The assertion line of every carried traceback, counted as lines rather than as substrings:
    // the same text also appears once per failure in the summary section, carried or not, so a
    // substring count over the whole output measures both and is fragile arithmetic.
    let assertion_lines = condensed
        .lines()
        .filter(|line| line.starts_with("E "))
        .count();
    assert_eq!(assertion_lines, carried);
}

#[test]
fn the_pytest_summary_survives_so_a_dropped_failure_is_still_named() {
    let condensed = condense_pair(
        &cargo_arguments("python -m pytest"),
        &failing_pytest(/*count*/ 30),
        Some(false),
    );

    // This is what makes capping cheap here: every failure, carried or not, is named with its
    // reason in the summary section.
    for index in 0..30 {
        assert!(
            condensed.contains(&format!("FAILED test_probe.py::test_case_{index}")),
            "failure {index} is missing from the summary:\n{condensed}"
        );
    }
    assert!(condensed.contains("30 failed, 2 passed"), "{condensed}");
}

#[test]
fn a_mypy_diagnostic_keeps_the_notes_attached_to_it() {
    let mut text = String::new();
    for index in 0..30 {
        text.push_str(&format!(
            "src/app/module_{index}.py:{index}: error: Incompatible return value type (got \"int\", expected \"str\")  [return-value]\n"
        ));
        text.push_str(&format!(
            "src/app/module_{index}.py:{index}: note: \"Thing\" defined here\n"
        ));
    }
    text.push_str("Found 30 errors in 30 files (checked 42 source files)");

    let condensed = condense_pair(&cargo_arguments("mypy src"), &text, Some(false));

    let carried = condensed.matches(": error:").count();
    assert_eq!(carried, MAX_ERROR_BLOCKS);
    // A note belongs to the diagnostic above it, so the two counts move together.
    assert_eq!(condensed.matches(": note:").count(), carried);
    assert!(
        condensed.contains("Found 30 errors in 30 files"),
        "{condensed}"
    );
}

#[test]
fn an_unrecognised_dialect_is_not_capped() {
    // `make` is on the allowlist, so its receipt can be dropped, but nothing here parses its
    // diagnostics and a line that merely starts with `error:` is not a promise of a grammar.
    let mut text = String::new();
    for index in 0..40 {
        text.push_str(&format!("error: something went wrong in step {index}\n"));
    }
    let text = text.trim_end().to_string();

    let condensed = condense_pair(&cargo_arguments("make all"), &text, Some(false));

    assert_eq!(condensed.matches("error: something went wrong").count(), 40);
}

#[test]
fn a_command_merely_containing_a_directory_name_still_gets_filtered() {
    // `printenv` contains "env" as a substring and names no directory at all. Under the substring
    // check its listing went unfiltered, as did anything mentioning `os.environ` or passing
    // `--environment`. `python -m venv` is *not* an example: `venv` is on the list, so that
    // command does name one.
    let text = listing(/*project_lines*/ 40, /*artifact_lines*/ 40);

    for command in ["printenv", "python -c \"print(os.environ)\""] {
        let arguments = serde_json::json!({ "command": command }).to_string();
        let report = report_for(&arguments, &text);
        assert_eq!(report.artifact_lines, 40, "{command} disabled the stage");
    }
}

#[test]
fn a_command_that_does_name_one_keeps_its_listing_whole() {
    // The guard's actual job: a deliberate look inside `node_modules` is the answer, not noise.
    let text = listing(/*project_lines*/ 40, /*artifact_lines*/ 40);
    let arguments = serde_json::json!({ "command": "ls node_modules" }).to_string();

    assert_eq!(condense_pair(&arguments, &text, Some(true)), text);
}

#[test]
fn a_dotted_directory_is_not_matched_by_a_shorter_one() {
    // `.vs` and `.vscode` are both on the list, and `.vs` is a prefix of the other.
    let text = listing(/*project_lines*/ 40, /*artifact_lines*/ 40);
    let arguments = serde_json::json!({ "command": "cat .vscode/settings.json" }).to_string();

    let report = report_for(&arguments, &text);

    assert_eq!(report.artifact_lines, 0, "naming .vscode disables the stage");
}

#[test]
fn shrinks_a_successful_allowlisted_command() {
    let arguments = serde_json::json!({ "command": "cargo build --release" }).to_string();
    let shrunk = condense_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");
    // The head and tail survive, so the model can still tell what ran and how it ended.
    assert!(shrunk.starts_with("first\n"), "{shrunk}");
    assert!(shrunk.ends_with("\nlast"), "{shrunk}");
    assert!(shrunk.len() < noisy_output().len());
}

#[test]
fn keeps_failed_output_whole() {
    let arguments = serde_json::json!({ "command": "cargo build" }).to_string();
    let text = noisy_output();
    assert_eq!(condense_pair(&arguments, &text, Some(false)), text);
    assert_eq!(condense_pair(&arguments, &text, None), text);
}

#[test]
fn keeps_short_output_whole() {
    let arguments = serde_json::json!({ "command": "cargo build" }).to_string();
    let text = "one\ntwo\nthree";
    assert_eq!(condense_pair(&arguments, text, Some(true)), text);
}

#[test]
fn keeps_commands_that_are_not_allowlisted() {
    for command in [
        "git status",
        "git log --oneline",
        "git diff HEAD",
        "cargo tree",
        "python script.py",
        "gh pr view 12",
        "ls -la",
    ] {
        let shrunk = condense_command(command);
        assert!(!was_shrunk(&shrunk), "{command} should not be shrunk");
    }
}

#[test]
fn shrinks_common_build_and_test_commands() {
    for command in [
        "cargo test -p codex-core",
        "npm install",
        "pnpm run build",
        "pytest -q",
        "go build ./...",
        "make -j8",
        "docker build .",
        "git push origin main",
        "/usr/local/bin/cargo check",
        "cargo.exe clippy",
        "env RUST_LOG=debug cargo nextest run",
        "CI=1 npm test",
        "eslint src",
    ] {
        let shrunk = condense_command(command);
        assert!(was_shrunk(&shrunk), "{command} should be shrunk");
    }
}

#[test]
fn refuses_chained_or_redirected_command_lines() {
    for command in [
        "cargo build && ./target/debug/server",
        "cargo build | tee log.txt",
        "cargo build > out.txt",
        "cargo build; cat out.txt",
        "cargo build $(echo --release)",
    ] {
        let shrunk = condense_command(command);
        assert!(!was_shrunk(&shrunk), "{command} should not be shrunk");
    }
}

#[test]
fn unwraps_a_shell_wrapper_argv() {
    let arguments =
        serde_json::json!({ "command": ["bash", "-lc", "cargo build --release"] }).to_string();
    let shrunk = condense_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");

    let arguments = serde_json::json!({ "command": ["bash", "-lc", "git status"] }).to_string();
    let shrunk = condense_pair(&arguments, &noisy_output(), Some(true));
    assert!(!was_shrunk(&shrunk), "{shrunk}");
}

#[test]
fn accepts_a_plain_argv_command() {
    let arguments = serde_json::json!({ "command": ["cargo", "test", "--all"] }).to_string();
    let shrunk = condense_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");
}

#[test]
fn marker_marks_the_gap_and_nothing_else() {
    let shrunk = condense_command("cargo build");
    // The marker only marks the gap. Who trimmed it, how much, and that re-running is
    // identical are all in the header one screen above; repeating any of it here would be
    // paid twice on every request that carries this output.
    assert!(was_shrunk(&shrunk), "{shrunk}");
    assert!(shrunk.contains("[system] 167 lines trimmed"), "{shrunk}");
    assert!(!shrunk.contains("re-running"), "{shrunk}");
}

#[test]
fn polls_of_a_running_session_keep_the_head_and_tail() {
    // A `write_stdin` poll carries no command, so the allowlist can never match it; the gate
    // is the live process. One measured poll returned 45,319 tokens of server log whose only
    // useful line -- the traceback the model then fixed -- was the last one.
    let arguments = serde_json::json!({ "session_id": 20885, "chars": "" }).to_string();
    let mut log: Vec<String> = (0..200)
        .map(|index| format!("INFO: 127.0.0.1:{index} - \"POST /api/auth/login HTTP/1.1\" 200 OK"))
        .collect();
    log.push("sqlite3.ProgrammingError: objects created in a thread".to_string());
    let text = log.join("\n");

    let (condensed, report) =
        condense_exec_output(&arguments, /*exit_code*/ None, Some(20885), &text)
            .expect("a live session poll is shrinkable");
    assert!(was_shrunk(&condensed), "{condensed}");
    assert!(condensed.ends_with("sqlite3.ProgrammingError: objects created in a thread"));
    assert!(report.middle_lines > 0);
    assert!(condensed.len() < text.len());

    // Without a live process there is nothing known to be running, so nothing is assumed.
    assert_eq!(
        condense_exec_output(&arguments, /*exit_code*/ None, None, &text),
        None
    );
}

#[test]
fn strips_escape_sequences_but_keeps_column_padding() {
    // The padding stays: a run of spaces is one or two tokens, so removing it buys almost
    // nothing and rewrites text the model may be about to quote back in a patch.
    let text = "\u{1b}[32mINFO\u{1b}[0m   ready   \nplain line";
    let (condensed, report) =
        condense_exec_output("{}", Some(0), None, text).expect("noise is removed");
    assert_eq!(condensed, "INFO   ready   \nplain line");
    assert_eq!(report.escape_sequences, 2);
    // Only escapes were touched, and those have their own counter.
    assert_eq!(report.rewritten_lines, 0);
    // Nothing was lost, so nothing is announced: a line of explanation on each affected output
    // would cost more than the lossless stage removes from all of them.
    assert_eq!(report.summary(), None);
}

#[test]
fn leaves_trailing_whitespace_alone() {
    // Trailing whitespace is the only difference here, so there is nothing to condense at all.
    assert_eq!(
        condense_exec_output("{}", Some(0), None, "value       \nother    "),
        None
    );
}

#[test]
fn keeps_windows_line_endings_intact() {
    // A trailing carriage return ends the line; treating it as an overwrite separator would
    // leave the empty string after it and delete every line of Windows output. Measured
    // against real rollouts this bug looked like a 75.9% saving.
    let text = "first line\r\nsecond line\r\nthird line";
    let (condensed, _) =
        condense_exec_output("{}", Some(0), None, text).expect("the terminators are removed");
    assert_eq!(condensed, "first line\nsecond line\nthird line");
}

#[test]
fn keeps_only_the_last_state_of_a_carriage_return_overwrite() {
    // Deleting the carriage return instead would splice every discarded state into one line.
    let text = "10%\r50%\r100% done\nnext";
    let (condensed, report) =
        condense_exec_output("{}", Some(0), None, text).expect("the overwrite is removed");
    assert_eq!(condensed, "100% done\nnext");
    assert_eq!(report.rewritten_lines, 1);
}

#[test]
fn leaves_clean_output_untouched() {
    // Nothing removed means no copy and no notice in the header.
    assert_eq!(condense_exec_output("{}", Some(0), None, "one\ntwo"), None);
}

#[test]
fn drops_generated_directory_lines_from_a_listing() {
    let arguments = serde_json::json!({ "cmd": "Get-ChildItem -Recurse app" }).to_string();
    let text = listing(/*project*/ 10, /*artifact*/ 20);
    let report = report_for(&arguments, &text);
    assert_eq!(report.artifact_lines, 20);

    let (condensed, _) = condense_exec_output(&arguments, Some(0), None, &text).expect("filtered");
    assert!(!condensed.contains("node_modules"), "{condensed}");
    assert!(condensed.contains("page-0.tsx"), "{condensed}");
    assert!(condensed.contains("page-9.tsx"), "{condensed}");
}

#[test]
fn matches_path_components_rather_than_the_text_of_a_line() {
    // A listing of ignore rules names these directories without being one.
    let arguments = serde_json::json!({ "cmd": "Get-Content .gitignore" }).to_string();
    let text = (0..20)
        .map(|index| format!("dist-{index}/"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(report_for(&arguments, &text).artifact_lines, 0);
}

#[test]
fn keeps_a_listing_the_command_asked_for() {
    // If the model went looking inside the dependency tree, the answer is the dependency tree.
    let arguments =
        serde_json::json!({ "cmd": "Get-ChildItem -Recurse app/node_modules" }).to_string();
    let text = listing(/*project*/ 10, /*artifact*/ 20);
    assert_eq!(report_for(&arguments, &text).artifact_lines, 0);
}

#[test]
fn keeps_a_listing_with_too_few_artifact_lines() {
    let arguments = serde_json::json!({ "cmd": "Get-ChildItem -Recurse app" }).to_string();
    let text = listing(/*project*/ 20, /*artifact*/ 5);
    assert_eq!(report_for(&arguments, &text).artifact_lines, 0);
}

#[test]
fn keeps_a_listing_that_is_almost_entirely_artifacts() {
    let arguments = serde_json::json!({ "cmd": "Get-ChildItem -Recurse app" }).to_string();
    let text = listing(/*project*/ 1, /*artifact*/ 19);
    assert_eq!(report_for(&arguments, &text).artifact_lines, 0);
}

#[test]
fn summary_names_every_cause_and_the_size() {
    let arguments = serde_json::json!({ "cmd": "Get-ChildItem -Recurse app" }).to_string();
    let mut text = listing(/*project*/ 10, /*artifact*/ 20);
    text.push_str("\r\n\u{1b}[32mdone\u{1b}[0m   ");

    let report = report_for(&arguments, &text);
    let summary = report.summary().expect("something was removed");
    assert!(
        summary.contains("Harness trimmed 20 generated-directory lines"),
        "{summary}"
    );
    assert!(summary.contains("re-running is identical"), "{summary}");
    assert!(report.removed_tokens > 0);
    // Lossless causes were counted but are deliberately absent from the notice.
    assert_eq!(report.escape_sequences, 2);
    assert_eq!(report.rewritten_lines, 1);
    assert!(!summary.contains("escape"), "{summary}");
    // The trailing spaces on that line survive the pass untouched.
    assert!(summary.contains("generated-directory"), "{summary}");

    assert_eq!(CondenseReport::default().summary(), None);
}

#[test]
fn reads_the_command_from_either_shell_tool() {
    // `shell` sends `command`; `exec_command` sends `cmd`. Both have to match.
    assert!(command_is_shrinkable(r#"{"command":"cargo build"}"#));
    assert!(command_is_shrinkable(
        r#"{"cmd":"git commit -m x","workdir":"/tmp"}"#
    ));
    assert!(!command_is_shrinkable(r#"{"cmd":"Get-Content big.log"}"#));
}

#[test]
fn leaves_a_short_lined_output_alone_even_when_it_is_long() {
    // Fifty one-word lines clear every line threshold and still amount to less than the
    // notice that would explain the cut. A `git commit` listing changed files looks like this.
    let text = (0..60)
        .map(|index| format!("M f{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let arguments = serde_json::json!({ "cmd": "git commit -m x" }).to_string();
    let condensed = condense_pair(&arguments, &text, Some(true));
    assert_eq!(condensed, text);

    // The same command with real output to remove is still shrunk. The lines are what `git
    // commit` actually prints - a build receipt here would be removed by the progress filter
    // before shrinking could be the thing under test.
    let big = (0..60)
        .map(|index| format!("M  crates/some-fairly-long-package/src/module-{index}.rs"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(was_shrunk(&condense_pair(&arguments, &big, Some(true))));
}

#[test]
fn leaves_a_listing_alone_when_its_artifact_lines_are_tiny() {
    let mut lines: Vec<String> = (0..10).map(|i| format!("src/a{i}.ts")).collect();
    lines.extend((0..15).map(|i| format!("node_modules/{i}")));
    let text = lines.join("\n");
    let arguments = serde_json::json!({ "cmd": "Get-ChildItem -Recurse app" }).to_string();
    assert_eq!(report_for(&arguments, &text).artifact_lines, 0);
}

#[test]
fn the_lossless_pass_removes_noise_and_keeps_every_other_line() {
    // What code mode gets. `cargo build` is on the shrink allowlist and this is well over the
    // line threshold, so the full pass would cut the middle out; the lossless one must not.
    let mut lines = vec![
        "warning: in the working copy of 'a.py', LF will be replaced by CRLF the next time Git touches it"
            .to_string(),
        "\u{1b}[32mCompiling\u{1b}[0m start".to_string(),
    ];
    // Not a build receipt, for the reason given on `noisy_output`: the subject at the end of this
    // test is head/tail shrinking, and the progress filter would remove a receipt before it ran.
    lines.extend((0..200).map(|index| format!("note: detail line {index}")));
    let text = lines.join("\n");

    let (condensed, report) = condense_exec_output_lossless(&text).expect("noise is removed");
    assert!(!condensed.contains("will be replaced by"), "{condensed}");
    assert_eq!(report.line_ending_warnings, 1);
    assert_eq!(report.escape_sequences, 2);
    // Every line but the warning survives, and nothing is announced.
    assert_eq!(condensed.lines().count(), 201);
    assert_eq!(report.summary(), None);
    assert_eq!(report.removed_tokens, 0);

    // The same input through the full pass loses its middle, which is the difference.
    let arguments = serde_json::json!({ "cmd": "cargo build" }).to_string();
    assert!(was_shrunk(&condense_pair(&arguments, &text, Some(true))));
}

#[test]
fn the_lossless_pass_leaves_clean_output_alone() {
    // `None` means "nothing was removed", which is what lets the caller keep its own string.
    assert_eq!(condense_exec_output_lossless("one\ntwo\nthree"), None);
}

#[test]
fn strips_git_line_ending_warnings() {
    // Both directions: `core.autocrlf=true` writes the first, `input` writes the second. The
    // path is whatever lies between the quotes, so an apostrophe in a filename does not end it
    // early -- git does not escape the inner quote.
    let arguments = serde_json::json!({ "cmd": "git checkout main" }).to_string();
    let text = [
        "Switched to branch 'main'",
        "warning: in the working copy of 'tests/test_query.py', LF will be replaced by CRLF the next time Git touches it",
        "warning: in the working copy of 'src/app.py', CRLF will be replaced by LF the next time Git touches it",
        "warning: in the working copy of 'don't-touch.py', LF will be replaced by CRLF the next time Git touches it",
        "Your branch is up to date with 'origin/main'.",
    ]
    .join("\n");

    let (condensed, report) = condense_exec_output(&arguments, Some(0), /*process_id*/ None, &text)
        .expect("both directions of the warning are removed");
    assert_eq!(
        condensed,
        "Switched to branch 'main'\nYour branch is up to date with 'origin/main'."
    );
    assert_eq!(report.line_ending_warnings, 3);
}

#[test]
fn removes_a_warnings_only_output_without_announcing_it() {
    // Counting is what makes this work at all: a stage that rewrote the text without touching
    // the report would leave `is_empty()` true, and the caller answers an empty report by
    // falling back to the original bytes -- warnings and all. On Windows `normalize` would have
    // set `rewritten_lines` and hidden that bug; this fixture is deliberately LF-only, which is
    // also what `core.autocrlf=input` produces on the platforms that use it.
    let arguments = serde_json::json!({ "cmd": "git add -A" }).to_string();
    let text = [
        "warning: in the working copy of 'a.py', LF will be replaced by CRLF the next time Git touches it",
        "warning: in the working copy of 'b.py', LF will be replaced by CRLF the next time Git touches it",
    ]
    .join("\n");

    let (condensed, report) = condense_exec_output(&arguments, Some(0), /*process_id*/ None, &text)
        .expect("a warnings-only output is still condensed");
    // `git add` prints nothing on success, so nothing is what is left; the header still carries
    // `Process exited with code 0`.
    assert_eq!(condensed, "");
    assert_eq!(report.line_ending_warnings, 2);
    // Nothing was lost, so nothing is announced -- and the notice would read `~0 tokens` anyway,
    // because the strip happens before the size is measured.
    assert_eq!(report.summary(), None);
    assert_eq!(report.removed_tokens, 0);
}

#[test]
fn keeps_an_unrelated_warning_line() {
    // The match is the whole sentence, not the word `warning`. A compiler diagnostic is the most
    // valuable thing in a failing build's output and shares only its first token; the middle line
    // here is the near miss, carrying the phrase without git's shape around it.
    let text = [
        "warning: unused variable: `path`",
        "warning: in the working copy, LF will be replaced by CRLF",
        "warning: 1 warning emitted",
    ]
    .join("\n");
    assert_eq!(
        condense_exec_output("{}", Some(0), /*process_id*/ None, &text),
        None
    );
}

#[test]
fn keeps_a_line_ending_warning_the_command_went_looking_for() {
    // Matching the line end to end is the gate, which is why there is none on the arguments. A
    // model that searched for this text gets it back with a `path:line:` prefix, a shape git
    // never writes, so the hit survives without anyone having to parse the command.
    let arguments = serde_json::json!({ "cmd": "rg 'will be replaced by' notes.md" }).to_string();
    let text = "notes.md:3:warning: in the working copy of 'a.py', LF will be replaced by CRLF the next time Git touches it";
    assert_eq!(
        condense_exec_output(&arguments, Some(0), /*process_id*/ None, text),
        None
    );
}

#[test]
fn strips_line_ending_warnings_that_arrived_with_windows_terminators() {
    // Runs after `normalize` for exactly this reason: on the platform that produces these
    // warnings they arrive as `...touches it\r\n`, and the match is anchored to the end of the
    // line. Reversing the two stages would match nothing where it matters most.
    let text = "warning: in the working copy of 'a.py', LF will be replaced by CRLF the next time Git touches it\r\nSwitched to branch 'main'\r\n";
    let (condensed, report) = condense_exec_output("{}", Some(0), /*process_id*/ None, text)
        .expect("the warning is removed");
    assert_eq!(condensed, "Switched to branch 'main'\n");
    assert_eq!(report.line_ending_warnings, 1);
}

#[test]
fn counts_line_ending_warnings_apart_from_what_the_notice_announces() {
    // Stripped before the size is measured, so their tokens are never attributed to a lossy
    // cause: `Harness trimmed 20 generated-directory lines (~T tokens)` has to mean those twenty
    // lines, which is only true if T is the same with and without the warnings.
    let arguments = serde_json::json!({ "cmd": "Get-ChildItem -Recurse app" }).to_string();
    let warnings: Vec<String> = (0..4)
        .map(|index| {
            format!(
                "warning: in the working copy of 'src/page-{index}.tsx', LF will be replaced by CRLF the next time Git touches it"
            )
        })
        .collect();
    let text = format!(
        "{}\n{}",
        warnings.join("\n"),
        listing(/*project*/ 10, /*artifact*/ 20)
    );

    let report = report_for(&arguments, &text);
    assert_eq!(report.line_ending_warnings, 4);
    assert_eq!(report.artifact_lines, 20);
    assert_eq!(
        report.removed_tokens,
        report_for(&arguments, &listing(/*project*/ 10, /*artifact*/ 20)).removed_tokens
    );
}
