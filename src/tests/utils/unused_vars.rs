//! Tests for optimizer/unused_vars.rs

use crate::optimizer::unused_vars::UnusedVariablesMetadata;

#[cfg(test)]
mod unused_vars_tests {
    use super::*;

    #[test]
    fn test_empty_metadata_returns_false() {
        // Test that empty metadata returns false for is_var_used
        let mut metadata = UnusedVariablesMetadata::default();
        assert!(!metadata.is_var_used("any_var".to_string()));
    }

    #[test]
    fn test_metadata_default_initialized() {
        // Test that metadata can be default initialized
        let mut metadata = UnusedVariablesMetadata::default();
        assert!(!metadata.is_var_used("test".to_string()));
    }

    #[test]
    fn test_multiple_calls_consistent() {
        // Test that multiple calls return consistent results
        let mut metadata = UnusedVariablesMetadata::default();
        assert!(!metadata.is_var_used("var1".to_string()));
        assert!(!metadata.is_var_used("var2".to_string()));
        assert!(!metadata.is_var_used("var3".to_string()));
    }
}
