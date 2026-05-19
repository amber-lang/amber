//! Test fixtures and helper functions for Amber tests
//!
//! This module provides reusable fixtures and helper functions for
//! creating test data and performing common test operations.

use crate::compiler::{AmberCompiler, CompilerOptions};

/// Create a new AmberCompiler instance with default test settings
pub fn create_compiler() -> AmberCompiler {
    let options = CompilerOptions::default();
    AmberCompiler::new(String::new(), None, options)
}

/// Compile a snippet of Amber code and return the result
/// 
/// This helper is useful for quick compilation tests without
/// needing to create actual files.
pub fn compile_snippet(code: &str) -> Result<String, String> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    compiler
        .compile()
        .map(|(_, code)| code)
        .map_err(|e| e.message.unwrap_or_else(|| "Unknown error".to_string()))
}

/// Parse and compile Amber code, returning the optimized output
/// 
/// This helper mirrors the standard test flow of compiling Amber code
/// and returning the generated output.
pub fn translate_and_optimize(code: &str) -> Result<String, String> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    compiler
        .compile()
        .map(|(_, code)| code)
        .map_err(|e| e.message.unwrap_or_else(|| "Unknown error".to_string()))
}

/// Helper to create test fixtures for common data structures
pub mod test_fixtures {
    /// Create a simple variable assignment snippet
    pub fn var_assignment(name: &str, value: &str) -> String {
        format!("{} = {};", name, value)
    }

    /// Create a function call snippet
    pub fn function_call(name: &str, args: &[&str]) -> String {
        let args_str = args.join(", ");
        format!("{}({});", name, args_str)
    }

    /// Create a block snippet with statements
    pub fn block(statements: &[&str]) -> String {
        format!("{{\n  {}\n}}", statements.join("\n  "))
    }
}

pub use test_fixtures::*;
