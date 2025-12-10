use assert_cmd::Command; // Fixes the mismatched type error by using the correct struct
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

// --- Helper for creating a command instance ---
fn cmd() -> Command {
    // This is the correct, non-deprecated way to instantiate the command,
    // converting the std::process::Command returned by the macro into
    // the assert_cmd::Command struct required for all test methods.
    Command::from(cargo_bin_cmd!("prepend"))
}

// --- End-to-End Tests ---

#[test]
fn test_cli_help_flag() {
    let mut cmd = cmd();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--help"));
}

#[test]
fn test_cli_no_arguments() {
    // Should display help and exit successfully if no arguments are provided.
    let mut cmd = cmd();

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_dry_run_mode() {
    let file = NamedTempFile::new().unwrap();
    let original_content = "Original content\n";
    fs::write(file.path(), original_content).unwrap();

    let mut cmd = cmd();
    cmd.arg("--dry-run").arg(file.path()).arg("Header text");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[33mDRY-RUN MODE:\x1b[0m"))
        .stdout(predicate::str::contains("Header text"));

    // Crucial assertion: file must not be modified
    let content = fs::read_to_string(file.path()).unwrap();
    assert_eq!(content, original_content);
}

#[test]
fn test_cli_prepend_with_argument() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "Original\n").unwrap();

    let mut cmd = cmd();
    cmd.arg(file.path()).arg("New Header");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[32mSUCCESS:\x1b[0m"));

    let content = fs::read_to_string(file.path()).unwrap();
    assert_eq!(content, "New Header\nOriginal\n");
}

#[test]
fn test_cli_nonexistent_file() {
    let mut cmd = cmd();
    cmd.arg("/nonexistent/file.txt").arg("Header");

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("\x1b[31mERROR:\x1b[0m"))
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_cli_directory_instead_of_file() {
    let dir = TempDir::new().unwrap();

    let mut cmd = cmd();
    cmd.arg(dir.path()).arg("Header");

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("ERROR"))
        .stderr(predicate::str::contains("not a regular file"));
}

#[test]
fn test_cli_interactive_mode_with_stdin() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "Original\n").unwrap();

    let mut cmd = cmd();
    cmd.arg(file.path()).write_stdin("Interactive Header\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SUCCESS"));

    let content = fs::read_to_string(file.path()).unwrap();
    assert_eq!(content, "Interactive Header\nOriginal\n");
}

#[test]
fn test_cli_multiline_argument() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "Original\n").unwrap();

    let mut cmd = cmd();
    // The argument is passed as a single string, including newlines
    cmd.arg(file.path()).arg("Line 1\nLine 2\nLine 3");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SUCCESS"));

    let content = fs::read_to_string(file.path()).unwrap();
    // The program should append one final newline after the entire argument string
    assert_eq!(content, "Line 1\nLine 2\nLine 3\nOriginal\n");
}

#[test]
fn test_cli_empty_stdin() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "Original\n").unwrap();

    let mut cmd = cmd();
    cmd.arg(file.path()).write_stdin("");

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("ERROR"))
        .stderr(predicate::str::contains("Input text is empty."));
}

#[test]
fn test_cli_uncommon_extension_warning() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.xyz");
    fs::write(&file_path, "content\n").unwrap();

    let mut cmd = cmd();
    cmd.arg(&file_path).arg("Header");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[33mWARNING:\x1b[0m"))
        .stdout(predicate::str::contains("Uncommon extension '.xyz'."));
}

#[test]
fn test_cli_successful_modification() {
    let file = NamedTempFile::new().unwrap();
    let original = "Original content\n";
    fs::write(file.path(), original).unwrap();

    let mut cmd = cmd();
    cmd.arg(file.path()).arg("Header");

    let content_before = fs::read_to_string(file.path()).unwrap();
    assert_eq!(content_before, original);

    cmd.assert().success();

    let content_after = fs::read_to_string(file.path()).unwrap();
    assert_ne!(content_before, content_after);
    assert!(content_after.starts_with("Header\n"));
    assert!(content_after.ends_with(original));
}

#[test]
fn test_cli_special_characters() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "Original\n").unwrap();

    let mut cmd = cmd();
    let special_text = "Special: 你好 🦀";
    cmd.arg(file.path()).arg(special_text);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SUCCESS"));

    let content = fs::read_to_string(file.path()).unwrap();
    // The program adds a newline after the argument
    assert!(content.starts_with(&format!("{}\n", special_text)));
}
