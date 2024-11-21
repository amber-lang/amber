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
