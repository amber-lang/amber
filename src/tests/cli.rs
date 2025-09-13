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

// Test that invalid escape sequences generate warnings
#[test]
fn invalid_escape_sequence_warning() {
    // Amber code
    let amber_code = r#"echo "\c""#;
    
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, _bash_code) = compiler.compile().unwrap();
    
    // Assert exactly one warning
    assert_eq!(messages.len(), 1);
    let warning_text = messages[0].message.clone().unwrap();
    assert!(warning_text.contains("Invalid escape sequence '\\c'"));
}

// Test that valid escape sequences don't generate warnings
#[test]
fn valid_escape_sequence_no_warning() {
    // Amber code
    let amber_code = r#"echo "\n\t\\""#;
    
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, _bash_code) = compiler.compile().unwrap();
    
    // Assert no warnings
    assert_eq!(messages.len(), 0);
}

// Test invalid escape sequence
#[test]
fn invalid_escape_sequence_x_warning() {
    // Amber code
    let amber_code = r#"echo "\x""#;
    
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    let (messages, _bash_code) = compiler.compile().unwrap();
    
    // Assert exactly one warning
    assert_eq!(messages.len(), 1);
    let warning_text = messages[0].message.clone().unwrap();
    assert!(warning_text.contains("Invalid escape sequence '\\x'"));
}
