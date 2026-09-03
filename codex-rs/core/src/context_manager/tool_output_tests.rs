use super::*;
use codex_protocol::models::FunctionCallOutputPayload;

const CALL_ID: &str = "call-1";

/// Output long enough to be worth shrinking, with a recognizable first and last line.
fn noisy_output() -> String {
    let mut lines = vec!["first".to_string()];
    for index in 0..200 {
        lines.push(format!("   Compiling crate-{index}"));
    }
    lines.push("last".to_string());
    lines.join("\n")
}

fn call(arguments: &str) -> ResponseItemEnvelope {
    ResponseItemEnvelope::new(ResponseItem::FunctionCall {
        id: None,
        name: "shell".to_string(),
        namespace: None,
        arguments: arguments.to_string(),
        encrypted_function_args: None,
        call_id: CALL_ID.to_string(),
        internal_chat_message_metadata_passthrough: None,
    })
}

fn output(text: &str, success: Option<bool>) -> ResponseItemEnvelope {
    ResponseItemEnvelope::new(ResponseItem::FunctionCallOutput {
        id: None,
        call_id: Some(CALL_ID.to_string()),
        name: None,
        namespace: None,
        output: FunctionCallOutputPayload {
            body: FunctionCallOutputBody::Text(text.to_string()),
            success,
        },
        internal_chat_message_metadata_passthrough: None,
    })
}

/// Runs the shrinker over one call/output pair and returns the resulting output text.
fn shrink_pair(arguments: &str, text: &str, success: Option<bool>) -> (String, ShrinkReport) {
    let mut items = vec![call(arguments), output(text, success)];
    let report = shrink_completed_outputs(&mut items, /*prefix_break*/ Some(0), |_| true);
    let ResponseItem::FunctionCallOutput { output, .. } = &items[1].item else {
        panic!("expected a function call output");
    };
    let FunctionCallOutputBody::Text(text) = &output.body else {
        panic!("expected a text body");
    };
    (text.clone(), report)
}

fn shrink_command(command: &str) -> String {
    let arguments = serde_json::json!({ "command": command }).to_string();
    shrink_pair(&arguments, &noisy_output(), Some(true)).0
}

fn was_shrunk(text: &str) -> bool {
    text.contains("[system] exit 0")
}

#[test]
fn shrinks_a_successful_allowlisted_command_and_reports_what_it_removed() {
    let arguments = serde_json::json!({ "command": "cargo build --release" }).to_string();
    let (shrunk, report) = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");
    // The head and tail survive, so the model can still tell what ran and how it ended.
    assert!(shrunk.starts_with("first\n"), "{shrunk}");
    assert!(shrunk.ends_with("\nlast"), "{shrunk}");
    assert!(shrunk.len() < noisy_output().len());
    assert_eq!(report.outputs, 1);
    assert_eq!(report.lines_removed, 167);
    assert_eq!(report.call_ids, vec![CALL_ID.to_string()]);
    assert!(report.tokens_removed > 0, "{report:?}");
    assert_eq!(report.shrinkable_before_break, 0);

    // With nothing dropped there is no break to ride on, so the very same output is left whole
    // and only counted. That count is the size of the saving the cache gate declines to take.
    let mut items = vec![call(&arguments), output(&noisy_output(), Some(true))];
    let refused = shrink_completed_outputs(&mut items, /*prefix_break*/ None, |_| true);
    assert_eq!(refused.outputs, 0);
    assert_eq!(refused.shrinkable_before_break, 1);
}

#[test]
fn keeps_failed_output_whole() {
    let arguments = serde_json::json!({ "command": "cargo build" }).to_string();
    let text = noisy_output();
    assert_eq!(shrink_pair(&arguments, &text, Some(false)).0, text);
    assert_eq!(shrink_pair(&arguments, &text, None).0, text);
}

#[test]
fn keeps_short_output_whole() {
    let arguments = serde_json::json!({ "command": "cargo build" }).to_string();
    let text = "one\ntwo\nthree";
    assert_eq!(shrink_pair(&arguments, text, Some(true)).0, text);
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
        let shrunk = shrink_command(command);
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
        let shrunk = shrink_command(command);
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
        let shrunk = shrink_command(command);
        assert!(!was_shrunk(&shrunk), "{command} should not be shrunk");
    }
}

#[test]
fn unwraps_a_shell_wrapper_argv() {
    let arguments =
        serde_json::json!({ "command": ["bash", "-lc", "cargo build --release"] }).to_string();
    let (shrunk, _) = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");

    let arguments = serde_json::json!({ "command": ["bash", "-lc", "git status"] }).to_string();
    let (shrunk, _) = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(!was_shrunk(&shrunk), "{shrunk}");
}

#[test]
fn accepts_a_plain_argv_command() {
    let arguments = serde_json::json!({ "command": ["cargo", "test", "--all"] }).to_string();
    let (shrunk, _) = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");
}

#[test]
fn leaves_output_from_the_running_turn_alone() {
    let text = noisy_output();
    let arguments = serde_json::json!({ "command": "cargo build" }).to_string();
    let mut items = vec![call(&arguments), output(&text, Some(true))];
    shrink_completed_outputs(&mut items, /*prefix_break*/ Some(0), |_| false);
    let ResponseItem::FunctionCallOutput { output, .. } = &items[1].item else {
        panic!("expected a function call output");
    };
    assert_eq!(output.body, FunctionCallOutputBody::Text(text));
}

#[test]
fn marker_reports_the_exit_status_and_the_actor() {
    let shrunk = shrink_command("cargo build");
    // A model that cannot tell the harness did this will run the command again.
    assert!(shrunk.contains("exit 0"), "{shrunk}");
    assert!(
        shrunk.contains("the harness trimmed this output, the command did not"),
        "{shrunk}"
    );
    assert!(shrunk.contains("re-running the command"), "{shrunk}");
    assert!(shrunk.contains("167 middle lines removed"), "{shrunk}");
}
