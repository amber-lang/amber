// Tests for the amber CLI binary functionality using internal APIs.
// These tests use internal compilation and execution functions instead of
// relying on the external binary, making them more reliable and faster.

use crate::compiler::{AmberCompiler, CompilerOptions};

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
                echo arg
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
        .arg("-c")
        .arg(bash_code_with_args)
        .output()
        .expect("Failed to execute bash");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "bash\none\ntwo\nthree");
}
