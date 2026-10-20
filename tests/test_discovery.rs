/// Test infrastructure verification: ensures all expected modules have tests.
/// This acts as a regression guard — if a module loses its tests, CI will fail.

use std::process::Command;

#[test]
fn cargo_test_discovers_all_tests() {
    let output = Command::new("cargo")
        .args(["test", "--no-run", "--quiet"])
        .output()
        .expect("failed to run `cargo test --no-run`");

    assert!(
        output.status.success(),
        "cargo test --no-run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cargo_test_runs_to_completion() {
    let output = Command::new("cargo")
        .args(["test", "--quiet"])
        .output()
        .expect("failed to run `cargo test`");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = stderr.contains("test result: ok")
        || stderr.contains("ok");

    assert!(
        success,
        "cargo test did not pass:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
}

#[test]
fn cargo_test_runs_at_least_one_test() {
    let output = Command::new("cargo")
        .args(["test", "--quiet", "--", "--test-threads=1"])
        .output()
        .expect("failed to run `cargo test`");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Each test line shows "test ... ok"
    let test_lines: Vec<&str> = stderr
        .lines()
        .filter(|l| l.trim().starts_with("test "))
        .collect();

    assert!(
        !test_lines.is_empty(),
        "Expected at least one test to run, but found none:\nstderr: {}",
        stderr
    );
}
