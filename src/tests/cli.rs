// Tests for the amber CLI binary functionality using internal APIs.
// These tests use internal compilation and execution functions instead of
// relying on the external binary, making them more reliable and faster.

use crate::compiler::{AmberCompiler, CompilerOptions};

// Helper function to assert no warnings
fn assert_no_warnings(amber_code: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    let (messages, _bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Assert no warnings
    assert_eq!(messages.len(), 0, "Expected no warnings, got {}: {:?}", messages.len(), messages);
    
    Ok(())
}

// Helper function to assert a single warning with expected message
fn assert_single_warning(amber_code: &str, expected_message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    let (messages, _bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Assert exactly one warning
    assert_eq!(messages.len(), 1, "Expected exactly 1 warning, got {}: {:?}", messages.len(), messages);
    
    // Assert the warning contains the expected message
    let warning_text = messages[0].message.clone().unwrap_or_default();
    assert!(warning_text.contains(expected_message), 
           "Expected warning to contain '{}', got: '{}'", expected_message, warning_text);
    
    Ok(())
}

// Test that the bash error code is forwarded to the exit code of amber.
#[test]
fn bash_error_exit_code() -> Result<(), Box<dyn std::error::Error>> {
    // Amber code
    let amber_code = r#"
        main {
            $ notexistingcommand $?
        }
        "#;
    
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    // Compile the amber code to bash
    let (messages, bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Assert that no compiler messages were generated
    assert_eq!(messages.len(), 0);
    
    // Execute the bash code and check the exit status
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    
    // Verify that the command failed with exit code 127 (command not found)
    assert_eq!(exit_status.code(), Some(127), "Expected exit code 127 for command not found");
    
    Ok(())
}

// Test that invalid escape sequences generate warnings
#[test]
fn invalid_escape_sequence_warning() -> Result<(), Box<dyn std::error::Error>> {
    // Amber code
    let amber_code = r#"echo "\c""#;
    
    // Amber compiler setup and parse + assertSingleWarning
    assert_single_warning(amber_code, "Invalid escape sequence '\\c'")?;
    
    Ok(())
}

// Test that valid escape sequences don't generate warnings
#[test]
fn valid_escape_sequence_no_warning() -> Result<(), Box<dyn std::error::Error>> {
    // Amber code
    let amber_code = r#"echo "\n\t\\""#;
    
    // Amber compiler setup and parse + assertNoWarnings
    assert_no_warnings(amber_code)?;
    
    Ok(())
}

// Test invalid escape sequence
#[test]
fn invalid_escape_sequence_x_warning() -> Result<(), Box<dyn std::error::Error>> {
    // Amber code
    let amber_code = r#"echo "\x""#;
    
    // Amber compiler setup and parse + assertSingleWarning
    assert_single_warning(amber_code, "Invalid escape sequence '\\x'")?;
    
    Ok(())
}
