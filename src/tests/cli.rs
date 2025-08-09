// Tests for the amber CLI binary functionality using internal APIs.
// These tests use internal compilation and execution functions instead of
// relying on the external binary, making them more reliable and faster.

use crate::compiler::{AmberCompiler, CompilerOptions};
use heraclitus_compiler::prelude::Message;

// Helper function to compile amber code and return messages and bash code
fn compile_amber_code(code: &str) -> Result<(Vec<Message>, String), Box<dyn std::error::Error>> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    
    compiler.compile()
        .map_err(|e| format!("Compilation failed: {}", e.message.unwrap_or_default()).into())
}

// Helper function to collect message text from compiler messages
fn collect_message_text(messages: &[Message]) -> Vec<String> {
    messages.iter()
        .map(|msg| msg.message.clone().unwrap_or_default())
        .collect()
}

// Helper function to assert that messages contain a warning with specific pattern
fn assert_contains_warning(messages: &[String], pattern: &str, context: &str) {
    let found = messages.iter().any(|msg| msg.contains(pattern));
    assert!(found, "Expected warning containing '{}' in {}, got messages: {:?}", pattern, context, messages);
}

// Helper function to assert that messages don't contain escape sequence warnings
fn assert_no_escape_warnings(messages: &[String]) {
    let has_escape_warnings = messages.iter()
        .any(|msg| msg.contains("Invalid escape sequence"));
    assert!(!has_escape_warnings, "Expected no escape sequence warnings, got messages: {:?}", messages);
}

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
    let amber_code = r#"echo "\c""#;
    
    // Compile the amber code and capture messages
    let (messages, bash_code) = compile_amber_code(amber_code)?;
    
    // Collect all message content to check for warnings
    let all_messages = collect_message_text(&messages);
    
    // Check that we got the expected warning message content
    assert_contains_warning(&all_messages, "Invalid escape sequence '\\c'", "invalid escape sequence test");
    
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
    
    // Compile the amber code and capture messages
    let (messages, bash_code) = compile_amber_code(amber_code)?;
    
    // Collect all message content to check for warnings
    let all_messages = collect_message_text(&messages);
    
    // Check that we got no warning messages related to escape sequences
    assert_no_escape_warnings(&all_messages);
    
    // Execute the bash code to verify it works
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    assert_eq!(exit_status.code(), Some(0), "Expected successful execution");
    
    Ok(())
}

// Test multiple invalid escape sequences
#[test]
fn multiple_invalid_escape_sequences() -> Result<(), Box<dyn std::error::Error>> {
    let amber_code = r#"echo "\x\y\z""#;
    
    // Compile the amber code and capture messages
    let (messages, bash_code) = compile_amber_code(amber_code)?;
    
    // Collect all message content to check for warnings
    let all_messages = collect_message_text(&messages);
    
    // Verify we have warnings for each invalid escape sequence
    assert_contains_warning(&all_messages, "Invalid escape sequence '\\x'", "multiple escape sequences test");
    assert_contains_warning(&all_messages, "Invalid escape sequence '\\y'", "multiple escape sequences test");
    assert_contains_warning(&all_messages, "Invalid escape sequence '\\z'", "multiple escape sequences test");
    
    // Execute the bash code to verify it still works
    let exit_status = AmberCompiler::execute(bash_code, vec![])?;
    assert_eq!(exit_status.code(), Some(0), "Expected successful execution");
    
    Ok(())
}
