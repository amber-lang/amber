// Tests for the amber CLI binary functionality using internal APIs.
// These tests use internal compilation and execution functions instead of
// relying on the external binary, making them more reliable and faster.

use crate::compiler::{AmberCompiler, CompilerOptions};

// Test that the bash error code is forwarded to the exit code of amber.
#[test]
fn bash_error_exit_code() -> Result<(), Box<dyn std::error::Error>> {
    let amber_code = r#"
        main {
            $ notexistingcommand $?
        }
        "#;
    
    // Create compiler with no postprocessors (equivalent to --no-proc *)
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    // Compile the amber code to bash
    let (messages, bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Show any compiler messages (shouldn't be any for this test)
    messages.iter().for_each(|m| m.show());
    
    // Execute the bash code and check the exit status
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    
    // Verify that the command failed with exit code 127 (command not found)
    assert_eq!(exit_status.code(), Some(127), "Expected exit code 127 for command not found");
    
    Ok(())
}

// Test that invalid escape sequences generate warnings
#[test]
fn invalid_escape_sequence_warning() -> Result<(), Box<dyn std::error::Error>> {
    let amber_code = r#"echo "\c""#;
    
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    // Compile the amber code and capture messages
    let (messages, bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Collect all message content to check for warnings
    let all_messages: Vec<String> = messages.iter()
        .map(|msg| msg.message.clone().unwrap_or_default())
        .collect();
    
    // Check that we got the expected warning message content
    let found_invalid_escape = all_messages.iter()
        .any(|msg| msg.contains("Invalid escape sequence '\\c'"));
    assert!(found_invalid_escape, "Expected warning about invalid escape sequence '\\c', got messages: {:?}", all_messages);
    
    // Verify we have at least one message (the warning)
    assert!(!all_messages.is_empty(), "Expected at least one warning message");
    
    // Execute the bash code to verify it still works and produces the expected output
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    assert_eq!(exit_status.code(), Some(0), "Expected successful execution");
    
    Ok(())
}

// Test that valid escape sequences don't generate warnings
#[test]
fn valid_escape_sequence_no_warning() -> Result<(), Box<dyn std::error::Error>> {
    let amber_code = r#"echo "\n\t\\""#;
    
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    // Compile the amber code and capture messages
    let (messages, bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Collect all message content to check for warnings
    let all_messages: Vec<String> = messages.iter()
        .map(|msg| msg.message.clone().unwrap_or_default())
        .collect();
    
    // Check that we got no warning messages related to escape sequences
    let has_escape_warnings = all_messages.iter()
        .any(|msg| msg.contains("Invalid escape sequence"));
    assert!(!has_escape_warnings, "Expected no escape sequence warnings, got messages: {:?}", all_messages);
    
    // Execute the bash code to verify it works
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    assert_eq!(exit_status.code(), Some(0), "Expected successful execution");
    
    Ok(())
}

// Test multiple invalid escape sequences
#[test]
fn multiple_invalid_escape_sequences() -> Result<(), Box<dyn std::error::Error>> {
    let amber_code = r#"echo "\x\y\z""#;
    
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);
    
    // Compile the amber code and capture messages
    let (messages, bash_code) = compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()))?;
    
    // Collect all message content to check for warnings
    let all_messages: Vec<String> = messages.iter()
        .map(|msg| msg.message.clone().unwrap_or_default())
        .collect();
    
    // Verify we have warnings for each invalid escape sequence
    let found_x_escape = all_messages.iter()
        .any(|msg| msg.contains("Invalid escape sequence '\\x'"));
    assert!(found_x_escape, "Expected warning about invalid escape sequence '\\x', got messages: {:?}", all_messages);
    
    let found_y_escape = all_messages.iter()
        .any(|msg| msg.contains("Invalid escape sequence '\\y'"));
    assert!(found_y_escape, "Expected warning about invalid escape sequence '\\y', got messages: {:?}", all_messages);
    
    let found_z_escape = all_messages.iter()
        .any(|msg| msg.contains("Invalid escape sequence '\\z'"));
    assert!(found_z_escape, "Expected warning about invalid escape sequence '\\z', got messages: {:?}", all_messages);
    
    // Execute the bash code to verify it still works
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    assert_eq!(exit_status.code(), Some(0), "Expected successful execution");
    
    Ok(())
}
