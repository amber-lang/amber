// Tests for the amber CLI binary functionality using internal APIs.
// These tests use internal compilation and execution functions instead of
// relying on the external binary, making them more reliable and faster.

use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::testing::get_tests_to_run;
use crate::TestCommand;
use std::path::PathBuf;

// Test that the bash error code is forwarded to the exit code of amber.
#[test]
fn bash_error_exit_code() {
    // Amber code
    let amber_code = r#"
        main {
            $ notexistingcommand $?
        }
        "#;

    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();

    // Assert no warnings
    assert_eq!(messages.len(), 0);

    // Execute the bash code and check the exit status
    let exit_status = AmberCompiler::execute(bash_code, vec![]).unwrap();
    assert_eq!(exit_status.code(), Some(127));
}

// Test that the main arguments are passed correctly
#[test]
fn main_args_passed_correctly() {
    // Amber code
    let amber_code = r#"
        main(args) {
            for arg in args {
                echo(arg)
            }
        }
        "#;

    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();

    // Assert no warnings
    assert_eq!(messages.len(), 0);

    // Prepend arguments to the bash code to simulate passing arguments
    // We use `set --` to set positional parameters
    let bash_code_with_args = format!("set -- one two three\n{}", bash_code);

    // Execute the bash code and check the output
    let output = AmberCompiler::find_shell(None)
        .expect("Failed to find shell")
        .arg("-c")
        .arg(bash_code_with_args)
        .output()
        .expect("Failed to execute shell");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[1], "one");
    assert_eq!(lines[2], "two");
    assert_eq!(lines[3], "three");
}

#[test]
fn test_filtering() {
    let input = PathBuf::from("src/tests/validity");

    // Filter by test name "foo"
    let command = TestCommand {
        input: input.clone(),
        args: vec![],
        no_proc: vec![],
        target: None,
        test_case: Some("foo".to_string()),
    };
    let tests = get_tests_to_run(&command)
        .map_err(|e| format!("{:?}", e))
        .expect("Failed to get tests");

    // Should find test 'foo' from test_named_syntax.ab
    let found = tests.iter().any(|(path, name, _)| {
        path.to_string_lossy().contains("test_named_syntax") && name == "foo"
    });
    assert!(found, "Should find test 'foo' in test_named_syntax.ab");

    // Also verify we filtered out everything that doesn't match "foo"
    for (path, name, _) in &tests {
        let display = if name.is_empty() {
            format!("{}", path.display())
        } else {
            format!("{} ({})", path.display(), name)
        };
        assert!(
            display.contains("foo"),
            "Test {} should contain 'foo'",
            display
        );
    }
}

#[test]
fn test_input_prompt_stdin() {
    // Amber code using input_prompt
    let amber_code = r#"
        import { input_prompt } from "std/env"
        
        main {
            const name = input_prompt("Enter name: ")
            echo("Hello {name}")
        }
        "#;

    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();

    // Assert no warnings
    if !messages.is_empty() {
        for msg in messages {
            println!("{:?}", msg);
        }
        panic!("Compilation failed with warnings/errors");
    }

    // Execute the bash code with stdin input
    // We pipe "World" into the process
    let mut child = AmberCompiler::find_shell(None)
        .expect("Failed to find shell")
        .arg("-c")
        .arg(bash_code)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn shell");

    {
        use std::io::Write;
        // We run this in a block to ensure stdin is closed after writing
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"World\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hello World"));
}

#[test]
fn test_input_hidden_stdin() {
    // Amber code using input_hidden
    let amber_code = r#"
        import { input_hidden } from "std/env"
        
        main {
            const secret = input_hidden("Enter secret: ")
            echo("Secret: {secret}")
        }
        "#;

    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();

    // Assert no warnings
    if !messages.is_empty() {
        for msg in messages {
            println!("{:?}", msg);
        }
        panic!("Compilation failed with warnings/errors");
    }

    // Execute the bash code with stdin input
    // We pipe "SecretCode" into the process
    let mut child = AmberCompiler::find_shell(None)
        .expect("Failed to find shell")
        .arg("-c")
        .arg(bash_code)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn shell");

    {
        use std::io::Write;
        // We run this in a block to ensure stdin is closed after writing
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"SecretCode\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Secret: SecretCode"));
}

#[test]
fn test_input_confirm_stdin() {
    // Amber code using input_confirm
    let amber_code = r#"
        import { input_confirm } from "std/env"
        
        main {
            if input_confirm("Continue?", false) {
                echo("Continued")
            } else {
                echo("Aborted")
            }
        }
        "#;

    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();

    // Assert no warnings
    if !messages.is_empty() {
        for msg in messages {
            println!("{:?}", msg);
        }
        panic!("Compilation failed with warnings/errors");
    }

    // Execute the bash code with stdin input
    // We pipe "y" into the process
    let mut child = AmberCompiler::find_shell(None)
        .expect("Failed to find shell")
        .arg("-c")
        .arg(bash_code)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn shell");

    {
        use std::io::Write;
        // We run this in a block to ensure stdin is closed after writing
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"y").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Continued"));
}

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::OnceLock;
use tempfile::NamedTempFile;

fn amber_bin() -> String {
    static BUILD_AMBER_BIN: OnceLock<()> = OnceLock::new();

    BUILD_AMBER_BIN.get_or_init(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = std::process::Command::new(cargo)
            .args(["build", "--quiet", "--bin", "amber"])
            .status()
            .expect("Failed to build amber binary for CLI tests");
        assert!(
            status.success(),
            "Failed to build amber binary for CLI tests"
        );
    });

    std::env::var("CARGO_BIN_EXE_AMBER").unwrap_or_else(|_| "target/debug/amber".to_string())
}

#[test]
fn test_cli_error_invalid_command() {
    let mut cmd = Command::new(amber_bin());
    cmd.arg("g-e").assert().failure().stderr(
        predicate::str::contains("Unknown command: g-e")
            .or(predicate::str::contains("File not found: g-e")),
    );
}

#[test]
fn test_cli_typo_suggestion() {
    let mut cmd = Command::new(amber_bin());
    cmd.arg("buid").assert().failure().stderr(
        predicate::str::contains("Unknown command: buid")
            .and(predicate::str::contains("Did you mean 'build'?")),
    );
}

#[test]
fn test_cli_file_starting_with_dash() {
    let mut cmd = Command::new(amber_bin());

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let amber_code = r#"
        main {
            echo("Hello from dash file")
        }
        "#;

    std::fs::write(temp_file.path(), amber_code).expect("Failed to write test file");

    let output = cmd.arg(temp_file.path()).assert().success();
    let _ = temp_file.close();

    output.stderr(predicate::str::contains("Hello from dash file").not());
}
#[test]
fn test_cli_no_arguments_shows_help() {
    let mut cmd = Command::new(amber_bin());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: amber"));
}

#[test]
fn test_cli_stdin_execution() {
    let mut cmd = Command::new(amber_bin());

    let amber_code = r#"
        main {
            echo("Hello from stdin")
        }
        "#;

    cmd.arg("-")
        .write_stdin(amber_code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from stdin"));
}

#[test]
fn test_cli_unknown_option_rejected() {
    let mut cmd = Command::new(amber_bin());
    cmd.arg("--unknown-option")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown"));
}

#[test]
fn test_cli_target_build_zsh_shebang() {
    let mut cmd = Command::new(amber_bin());
    cmd.args([
        "build",
        "src/tests/validity/hello_world.ab",
        "-",
        "--target",
        "zsh",
    ])
    .assert()
    .success()
    .stdout(predicate::str::starts_with("#!/usr/bin/env zsh"));
}

#[test]
fn test_cli_top_level_target_build_zsh_shebang() {
    let mut cmd = Command::new(amber_bin());
    cmd.args([
        "--target",
        "zsh",
        "build",
        "src/tests/validity/hello_world.ab",
        "-",
    ])
    .assert()
    .success()
    .stdout(predicate::str::starts_with("#!/usr/bin/env zsh"));
}

#[test]
fn test_cli_subcommand_target_overrides_top_level_target() {
    let mut cmd = Command::new(amber_bin());
    cmd.args([
        "--target",
        "bash",
        "build",
        "src/tests/validity/hello_world.ab",
        "-",
        "--target",
        "zsh",
    ])
    .assert()
    .success()
    .stdout(predicate::str::starts_with("#!/usr/bin/env zsh"));
}

#[test]
fn test_cli_target_build_ksh_shebang() {
    let mut cmd = Command::new(amber_bin());
    cmd.args([
        "build",
        "src/tests/validity/hello_world.ab",
        "-",
        "--target",
        "ksh",
    ])
    .assert()
    .success()
    .stdout(predicate::str::starts_with("#!/usr/bin/env ksh"));
}

#[test]
fn test_cli_target_invalid_value() {
    let mut cmd = Command::new(amber_bin());
    cmd.args([
        "build",
        "src/tests/validity/hello_world.ab",
        "-",
        "--target",
        "fish",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("invalid value 'fish' for '--target <TARGET>'"));
}

#[test]
fn test_cli_target_alias_bash_maps_to_bash_shebang() {
    let mut cmd = Command::new(amber_bin());
    cmd.args([
        "build",
        "src/tests/validity/hello_world.ab",
        "-",
        "--target",
        "bash",
    ])
    .assert()
    .success()
    .stdout(predicate::str::starts_with("#!/usr/bin/env bash"));
}

#[test]
fn test_cli_bash_32_variable_ref_runtime() {
    let mut cmd = Command::new(amber_bin());
    cmd.env("AMBER_SHELL", "/bin/bash")
        .args([
            "src/tests/validity/variable_ref_set_number.ab",
            "--target",
            "bash-3.2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("42\n42"));
}

#[test]
fn test_cli_bash_32_array_ref_runtime() {
    let mut cmd = Command::new(amber_bin());
    cmd.env("AMBER_SHELL", "/bin/bash")
        .args([
            "src/tests/validity/array_assign_by_ref.ab",
            "--target",
            "bash-3.2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 2 42 4 5\n1 2 42 4 5"));
}

#[test]
fn test_cli_bash_32_swap_ref_runtime() {
    let mut cmd = Command::new(amber_bin());
    cmd.env("AMBER_SHELL", "/bin/bash")
        .args([
            "src/tests/validity/function_ref_swap.ab",
            "--target",
            "bash-3.2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("24\n12"));
}

#[test]
fn test_cli_bash_32_nested_ref_runtime() {
    let mut cmd = Command::new(amber_bin());
    cmd.env("AMBER_SHELL", "/bin/bash")
        .args([
            "src/tests/validity/variable_ref_nested.ab",
            "--target",
            "bash-3.2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("John\n1\n2\n3"));
}
