//! The condenser run against output captured from real tools, not fixtures.
//!
//! Every fixture in `tool_output_tests.rs` is a guess about a format until something checks it,
//! and twice in one sitting a guess here was wrong: the sub-separators inside a pytest traceback
//! were missed, and a line threshold calibrated on cargo's per-crate receipt could never fire on
//! pytest's fixed four lines of session noise. These two samples were produced by running the
//! tools - `cargo build` over a crate with a type error, an unresolved name and a missing method,
//! and `pytest` over a file with an assertion failure, a `TypeError` and a recursive `ValueError`
//! - and pasted in unmodified.

use super::*;

/// `cargo build` on a crate that does not compile. Captured verbatim.
const REAL_CARGO_FAILURE: &str = r#"   Compiling probe v0.0.0 (C:\tmp\probe)
error[E0308]: mismatched types
 --> src\lib.rs:3:22
  |
3 |     let wrong: i32 = "not a number";
  |                ---   ^^^^^^^^^^^^^^ expected `i32`, found `&str`
  |                |
  |                expected due to this

error[E0425]: cannot find function `no_such_function` in this scope
 --> src\lib.rs:9:19
  |
9 |     let missing = no_such_function();
  |                   ^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no method named `this_method_does_not_exist` found for type `i32` in the current scope
  --> src\lib.rs:14:11
   |
14 |     value.this_method_does_not_exist()
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `i32`

Some errors have detailed explanations: E0308, E0425, E0599.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `probe` (lib) due to 3 previous errors"#;

/// `pytest` on a file with three failing tests. Captured verbatim, sub-separators included.
const REAL_PYTEST_FAILURE: &str = r#"============================= test session starts =============================
platform win32 -- Python 3.13.15, pytest-9.1.1, pluggy-1.6.0
rootdir: C:\tmp\pyprobe
plugins: anyio-4.15.1, typeguard-4.6.0
collected 5 items

test_probe.py ..FFF                                                      [100%]

================================== FAILURES ===================================
____________________________ test_assertion_fails _____________________________

    def test_assertion_fails():
>       assert compute() == 5
E       assert 4 == 5
E        +  where 4 = compute()

test_probe.py:18: AssertionError
___________________________ test_raises_type_error ____________________________

    def test_raises_type_error():
>       helper(["not", "a", "dict"])

test_probe.py:22:
_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

value = ['not', 'a', 'dict']

    def helper(value):
>       return value["missing"]
               ^^^^^^^^^^^^^^^^
E       TypeError: list indices must be integers or slices, not str

test_probe.py:6: TypeError
______________________________ test_deep_frames _______________________________

    def test_deep_frames():
        def inner(n):
            if n == 0:
                raise ValueError("bottom reached")
            return inner(n - 1)

>       inner(3)

test_probe.py:31:
_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _
test_probe.py:29: in inner
    return inner(n - 1)
           ^^^^^^^^^^^^
_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

n = 0

    def inner(n):
        if n == 0:
>           raise ValueError("bottom reached")
E           ValueError: bottom reached

test_probe.py:28: ValueError
=========================== short test summary info ===========================
FAILED test_probe.py::test_assertion_fails - assert 4 == 5
FAILED test_probe.py::test_raises_type_error - TypeError: list indices must b...
FAILED test_probe.py::test_deep_frames - ValueError: bottom reached
========================= 3 failed, 2 passed in 0.17s ========================="#;

fn condense_failure(command: &str, text: &str) -> (String, CondenseReport) {
    let arguments = serde_json::json!({ "command": command }).to_string();
    condense_exec_output(&arguments, Some(1), /*process_id*/ None, text)
        .unwrap_or_else(|| (text.to_string(), CondenseReport::default()))
}

#[test]
fn the_dialects_are_recognised_in_real_output() {
    // Three errors, three headers, and the closing `could not compile` is not one of them.
    let blocks = REAL_CARGO_FAILURE
        .lines()
        .filter(|line| block_start(line, DiagnosticDialect::Rustc).is_some())
        .count();
    assert_eq!(blocks, 3);

    // Three failures. The `_ _ _ _` lines inside two of them must not count, or one traceback
    // would be capped into pieces.
    let failures = REAL_PYTEST_FAILURE
        .lines()
        .filter(|line| block_start(line, DiagnosticDialect::Pytest).is_some())
        .count();
    assert_eq!(failures, 3);
}

#[test]
fn a_real_cargo_failure_loses_only_its_receipt() {
    let (condensed, report) = condense_failure("cargo build", REAL_CARGO_FAILURE);

    // One `Compiling` line is not worth a notice, so nothing is adopted and the model gets the
    // failure exactly as the compiler wrote it. Three errors is under every cap.
    assert_eq!(condensed, REAL_CARGO_FAILURE);
    assert_eq!(report.summary(), None);
}

/// A small real pytest failure arrives verbatim, and this records why rather than wishing
/// otherwise.
///
/// The four lines there is to remove - `platform`, `rootdir`, `plugins` and the dot line - come to
/// about 43 tokens, and the header line explaining their removal costs about 26. `MIN_LOSSY_TOKENS`
/// wants 120 before it will adopt a lossy candidate, so the candidate is dropped and the model
/// sees everything.
///
/// That is the threshold working, not failing: it was measured for stages that remove *content*,
/// and it is deliberately conservative. The cost of the conservatism is those 43 tokens on small
/// runs, where they are also least worth chasing. On a run big enough for the noise to matter the
/// filter fires - `a_pytest_run_loses_its_session_header_and_its_dot_line` covers that case.
#[test]
fn a_small_real_pytest_failure_is_not_worth_condensing() {
    let (condensed, report) = condense_failure("python -m pytest", REAL_PYTEST_FAILURE);

    assert_eq!(condensed, REAL_PYTEST_FAILURE);
    assert_eq!(report.summary(), None);
}

/// The same input, with the diagnostics it would take to make condensing worthwhile.
///
/// This is the assertion that matters: whatever the threshold decides, the three tracebacks are
/// carried whole when they are carried at all - assertion lines, frame separators and `file:line`
/// footers - and the environment boilerplate is what goes.
#[test]
fn a_real_pytest_traceback_survives_whole_once_condensing_is_worth_it() {
    // The same session over sixty test files rather than one. Those dot lines are what a real
    // suite of that size prints, and they are the volume that makes removing the boilerplate
    // beside them worth announcing.
    let extra: String = (0..60)
        .map(|index| {
            format!(
                "tests/test_module_{index}.py ....                                        [ {}%]\n",
                index * 100 / 60
            )
        })
        .collect();
    let padded = REAL_PYTEST_FAILURE.replace(
        "test_probe.py ..FFF                                                      [100%]\n",
        &format!(
            "test_probe.py ..FFF                                                      [100%]\n{extra}"
        ),
    );
    let (condensed, _) = condense_failure("python -m pytest", &padded);

    for noise in ["platform win32", "rootdir:", "plugins:", "[100%]"] {
        assert!(!condensed.contains(noise), "{noise} survived:\n{condensed}");
    }
    for kept in [
        "E       assert 4 == 5",
        "E       TypeError: list indices must be integers",
        "E           ValueError: bottom reached",
        "test_probe.py:28: ValueError",
    ] {
        assert!(condensed.contains(kept), "{kept} was dropped:\n{condensed}");
    }
    assert_eq!(
        condensed.lines().filter(|l| l.starts_with("_ _")).count(),
        3
    );
    assert!(condensed.contains("collected 5 items"), "{condensed}");
    assert!(condensed.contains("3 failed, 2 passed"), "{condensed}");
}
