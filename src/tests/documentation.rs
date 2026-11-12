//! Tests for docs/module.rs and function declaration docs

use crate::compiler::{AmberCompiler, CompilerOptions, SourcePath};
use crate::docs::module::DocumentationModule;
use crate::impl_documentation_noop;
use crate::utils::ParserMetadata;

// Create a test type that uses the macro
struct TestType;
impl_documentation_noop!(TestType);

#[cfg(test)]
mod documentation_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Library with one documented function that has a `### Usage` section,
    /// one documented function without it and one function without docs.
    const LIBRARY: &str = r#"
/// Adds two numbers.
///
/// ### Usage
/// ```ab
/// echo(add(1, 2))
/// ```
pub fun add(a: Int, b: Int): Int {
    return a + b
}

/// Greets someone.
pub fun greet(name: Text, greeting: Text = "Hello"): Text {
    return "{greeting}, {name}!"
}

pub fun undocumented(items: [Int]): Null? {
    fail 1
}
"#;

    /// Generates documentation of a mock `LIBRARY` saved as `std/text.ab`,
    fn document_library(usage: bool) -> String {
        let temp_dir = tempdir().unwrap();
        let std_dir = temp_dir.path().join("std");
        fs::create_dir(&std_dir).unwrap();
        let path = std_dir.join("text.ab");
        fs::write(&path, LIBRARY).unwrap();
        let source = SourcePath::file(path.to_string_lossy()).unwrap();
        let compiler = AmberCompiler::new(LIBRARY.to_string(), source, CompilerOptions::default());
        let tokens = compiler.tokenize().unwrap();
        let (block, meta) = compiler.parse(tokens).unwrap();
        let (block, mut meta) = compiler.typecheck(block, meta).unwrap();
        meta.doc_usage = usage;
        block.document(&meta)
    }

    #[test]
    fn test_documentation_noop_macro() {
        let _test = TestType;
        // Test that the trait method compiles and returns empty string
        // We cannot easily create ParserMetadata without full setup,
        // so we verify the macro-generated impl exists via compilation
        let _doc_fn: fn(&TestType, &ParserMetadata) -> String = TestType::document;
    }

    #[test]
    fn test_document_module_trait_exists() {
        let _test = TestType;
        // Verify the trait is implemented by checking it compiles
        let _doc_fn: fn(&TestType, &ParserMetadata) -> String = TestType::document;
    }

    #[test]
    fn test_function_signature_rendering() {
        let docs = document_library(false);
        assert!(docs.contains("## `add`\n"));
        assert!(docs.contains("pub fun add(a: Int, b: Int): Int"));
        assert!(docs.contains(r#"pub fun greet(name: Text, greeting: Text = "Hello"): Text"#));
        // Functions without a doc comment get just the signature
        assert!(
            docs.contains("## `undocumented`\n\n```ab\npub fun undocumented(items: [Int]): Null?")
        );
    }

    #[test]
    fn test_usage_import_after_usage_section() {
        let docs = document_library(true);
        // The import statement is inserted at the top of the existing usage example
        let expected = "### Usage\n```ab\nimport { add } from \"std/text\"\n\necho(add(1, 2))";
        assert!(docs.contains(expected), "Unexpected documentation:\n{docs}");
    }

    #[test]
    fn test_usage_import_appended_without_usage_section() {
        let docs = document_library(true);
        // Without a usage section, a code block with the import statement is appended
        let expected = "Greets someone.\n```ab\nimport { greet } from \"std/text\"\n```";
        assert!(docs.contains(expected), "Unexpected documentation:\n{docs}");
    }

    #[test]
    fn test_no_usage_import_without_flag() {
        let docs = document_library(false);
        assert!(
            !docs.contains("import {"),
            "Unexpected documentation:\n{docs}"
        );
    }

    #[test]
    fn test_usage_flag_rejected_for_non_stdlib_file() {
        let temp_dir = tempdir().unwrap();
        // Same file stem as a stdlib module, but not inside a `std` directory
        let path = temp_dir.path().join("text.ab");
        fs::write(&path, LIBRARY).unwrap();
        let source = SourcePath::file(path.to_string_lossy()).unwrap();
        let compiler = AmberCompiler::new(LIBRARY.to_string(), source, CompilerOptions::default());
        let err = compiler.generate_docs(Some("-".into()), true).unwrap_err();
        let message = err.message.unwrap_or_default();
        assert!(message.contains("'--usage' flag is only supported"), "Unexpected error: {message}");
    }
}
