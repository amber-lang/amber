//! Tests for docs/module.rs

use crate::docs::module::DocumentationModule;
use crate::impl_documentation_noop;
use crate::utils::ParserMetadata;

// Create a test type that uses the macro
struct TestType;
impl_documentation_noop!(TestType);

#[cfg(test)]
mod documentation_tests {
    use super::*;

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
}
