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
    let output = std::process::Command::new("bash")
        .arg("--norc")
        .arg("-c")
        .arg(bash_code_with_args)
        .output()
        .expect("Failed to execute bash");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "bash\none\ntwo\nthree");
}

#[test]
fn test_filtering() {
    let input = PathBuf::from("src/tests/validity");

    // Case 1: Filter by file name substring "named_syntax"
    // Expect "test_named_syntax.ab" to be included
    let command = TestCommand {
        input: input.clone(),
        args: vec!["named_syntax".to_string()],
        no_proc: vec![],
    };

    let tests = get_tests_to_run(&command)
        .map_err(|e| format!("{:?}", e))
        .expect("Failed to get tests");
    assert!(
        !tests.is_empty(),
        "Should find tests in test_named_syntax.ab"
    );
    for (path, _, _) in &tests {
        assert!(path.to_string_lossy().contains("named_syntax"));
    }

    // Case 2: Filter by test name "foo"
    let command = TestCommand {
        input: input.clone(),
        args: vec!["foo".to_string()],
        no_proc: vec![],
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
    let mut child = std::process::Command::new("bash")
        .arg("-c")
        .arg(bash_code)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn bash");

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
    let mut child = std::process::Command::new("bash")
        .arg("-c")
        .arg(bash_code)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn bash");

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
    let mut child = std::process::Command::new("bash")
        .arg("-c")
        .arg(bash_code)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn bash");

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
