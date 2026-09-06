use super::*;

/// Output long enough to be worth shrinking, with a recognizable first and last line.
fn noisy_output() -> String {
    let mut lines = vec!["first".to_string()];
    for index in 0..200 {
        lines.push(format!("   Compiling crate-{index}"));
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
        lines.push(format!("C:\\work\\app\\node_modules\\dep-{index}\\index.js"));
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

fn was_shrunk(text: &str) -> bool {
    text.contains("[system] ") && text.contains(" lines trimmed")
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
fn strips_escape_sequences_and_padding() {
    let text = "\u{1b}[32mINFO\u{1b}[0m   ready   \nplain line";
    let (condensed, report) =
        condense_exec_output("{}", Some(0), None, text).expect("noise is removed");
    assert_eq!(condensed, "INFO   ready\nplain line");
    assert_eq!(report.escape_sequences, 2);
    assert_eq!(report.rewritten_lines, 1);
    // Nothing was lost, so nothing is announced. 59% of measured outputs carry some padding;
    // a line of explanation on each would cost more than the padding removed from all of them.
    assert_eq!(report.summary(), None);
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
    text.push_str("\n\u{1b}[32mdone\u{1b}[0m   ");

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
    assert!(!summary.contains("padding"), "{summary}");

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

    // The same command with real output to remove is still shrunk.
    let big = (0..60)
        .map(|index| format!("   Compiling some-fairly-long-crate-name-{index} v0.1.0 (/w/{index})"))
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
