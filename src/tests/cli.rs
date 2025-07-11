// Tests for the amber CLI binary itself.
// Make sure to run `cargo build` before running these tests.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

// Test that the bash error code is forwarded to the exit code of amber.
#[test]
fn bash_error_exit_code() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let mut file = NamedTempFile::new()?;

    writeln!(
        file,
        r#"
        main {{
            $ notexistingcommand $?
        }}
        "#
    )?;

    // Changes locale to default to prevent locale-specific error messages.
    cmd.env("LC_ALL", "C")
        .arg("run")
        .arg("--no-proc")
        .arg("*")
        .arg(file.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("notexistingcommand: command not found"))
        .code(127);

    Ok(())
}

// Test that invalid escape sequences generate warnings
#[test]
fn invalid_escape_sequence_warning() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("eval")
        .arg(r#"echo "\c""#);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("WARN  Invalid escape sequence '\\c'"))
        .stderr(predicate::str::contains("Only these escape sequences are supported: \\n, \\t, \\r, \\0, \\{, \\$, \\', \\\", \\\\"))
        .stdout(predicate::str::contains("\\c"));

    Ok(())
}

// Test that valid escape sequences don't generate warnings
#[test]
fn valid_escape_sequence_no_warning() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("eval")
        .arg(r#"echo "\n\t\\""#);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("WARN").not());

    Ok(())
}

// Test multiple invalid escape sequences
#[test]
fn multiple_invalid_escape_sequences() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("eval")
        .arg(r#"echo "\x\y\z""#);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Invalid escape sequence '\\x'"))
        .stderr(predicate::str::contains("Invalid escape sequence '\\y'"))
        .stderr(predicate::str::contains("Invalid escape sequence '\\z'"));

    Ok(())
}
