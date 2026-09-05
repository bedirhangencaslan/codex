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

/// Runs the live shrinker over one call's arguments and output.
fn shrink_pair(arguments: &str, text: &str, success: Option<bool>) -> String {
    let exit_code = match success {
        Some(true) => Some(0),
        Some(false) => Some(1),
        None => None,
    };
    shrink_exec_output(arguments, exit_code, text).unwrap_or_else(|| text.to_string())
}

fn shrink_command(command: &str) -> String {
    let arguments = serde_json::json!({ "command": command }).to_string();
    shrink_pair(&arguments, &noisy_output(), Some(true))
}

fn was_shrunk(text: &str) -> bool {
    text.contains("[system] exit 0")
}

#[test]
fn shrinks_a_successful_allowlisted_command() {
    let arguments = serde_json::json!({ "command": "cargo build --release" }).to_string();
    let shrunk = shrink_pair(&arguments, &noisy_output(), Some(true));
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
    assert_eq!(shrink_pair(&arguments, &text, Some(false)), text);
    assert_eq!(shrink_pair(&arguments, &text, None), text);
}

#[test]
fn keeps_short_output_whole() {
    let arguments = serde_json::json!({ "command": "cargo build" }).to_string();
    let text = "one\ntwo\nthree";
    assert_eq!(shrink_pair(&arguments, text, Some(true)), text);
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
    let shrunk = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");

    let arguments = serde_json::json!({ "command": ["bash", "-lc", "git status"] }).to_string();
    let shrunk = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(!was_shrunk(&shrunk), "{shrunk}");
}

#[test]
fn accepts_a_plain_argv_command() {
    let arguments = serde_json::json!({ "command": ["cargo", "test", "--all"] }).to_string();
    let shrunk = shrink_pair(&arguments, &noisy_output(), Some(true));
    assert!(was_shrunk(&shrunk), "{shrunk}");
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
    assert!(shrunk.contains("167 middle lines"), "{shrunk}");
    // The size is named so a rollout can be priced without the sidecar.
    assert!(shrunk.contains("tokens) removed"), "{shrunk}");
}

#[test]
fn exec_output_is_shrunk_on_the_way_to_the_model() {
    let arguments = r#"{"command":"cargo build"}"#;
    let noisy = noisy_output();

    let shrunk = shrink_exec_output(arguments, Some(0), &noisy).expect("clean build is shrinkable");
    assert!(shrunk.len() < noisy.len(), "{shrunk}");
    assert!(shrunk.starts_with("first"), "{shrunk}");
    assert!(shrunk.ends_with("last"), "{shrunk}");

    // A failure's output is the most valuable thing in the context.
    assert_eq!(shrink_exec_output(arguments, Some(1), &noisy), None);
    // So is the output of a command the allowlist does not recognize.
    assert_eq!(
        shrink_exec_output(r#"{"command":"Get-Content big.log"}"#, Some(0), &noisy),
        None
    );
    // An interactive session reports no exit code, so nothing is known to have finished.
    assert_eq!(shrink_exec_output(arguments, None, &noisy), None);
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
