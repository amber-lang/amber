// Tests for the amber CLI binary functionality using internal APIs.
// These tests use internal compilation and execution functions instead of
// relying on the external binary, making them more reliable and faster.

use crate::compiler::{AmberCompiler, CompilerOptions};

// Helper function to assert multiple warnings with expected messages
fn assert_multiple_warnings(amber_code: &str, expected_messages: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    // Amber compiler setup and parse
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    let (messages, _bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Assert expected number of warnings
    assert_eq!(messages.len(), expected_messages.len(), 
               "Expected {} warnings, got {}: {:?}", expected_messages.len(), messages.len(), messages);
    
    // Assert each expected message is found
    let warning_texts: Vec<String> = messages.iter()
        .map(|msg| msg.message.clone().unwrap_or_default())
        .collect();
    
    for expected_message in expected_messages {
        let found = warning_texts.iter().any(|msg| msg.contains(expected_message));
        assert!(found, "Expected warning containing '{}', got messages: {:?}", expected_message, warning_texts);
    }
    
    Ok(())
}

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

// Test multiple invalid escape sequences
#[test]
fn multiple_invalid_escape_sequences() -> Result<(), Box<dyn std::error::Error>> {
    // Amber code
    let amber_code = r#"echo "\x\y\z""#;
    
    // Amber compiler setup and parse + assertMultipleWarnings
    assert_multiple_warnings(amber_code, &[
        "Invalid escape sequence '\\x'",
        "Invalid escape sequence '\\y'", 
        "Invalid escape sequence '\\z'"
    ])?;
    
    Ok(())
}
