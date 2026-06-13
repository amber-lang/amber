//! Tests for utils/mod.rs functions
//!
//! Tests for: pluralize, pretty_join, is_all_caps
//! Note: These are integration tests that complement the inline tests in src/utils/mod.rs

use crate::utils::{pluralize, pretty_join, is_all_caps};

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== pluralize tests ====================

    #[test]
    fn test_pluralize_singular() {
        // Test with amount = 1 (singular)
        assert_eq!(pluralize(1, "error", "errors"), "error");
        assert_eq!(pluralize(1, "file", "files"), "file");
        assert_eq!(pluralize(1, "warning", "warnings"), "warning");
    }

    #[test]
    fn test_pluralize_non_singular_amounts() {
        // Test the implementation's threshold behavior:
        // values greater than 1 use the plural form, while 0 remains singular.
        assert_eq!(pluralize(0, "error", "errors"), "error");  // 0 is not > 1
        assert_eq!(pluralize(2, "error", "errors"), "errors");
        assert_eq!(pluralize(10, "file", "files"), "files");
        assert_eq!(pluralize(100, "warning", "warnings"), "warnings");
    }

    // ==================== pretty_join tests ====================

    #[test]
    fn test_pretty_join_single_item() {
        // Test with single item - should return just that item
        assert_eq!(pretty_join(&["hello"], "and"), "hello");
        assert_eq!(pretty_join(&[42], "or"), "42");
    }

    #[test]
    fn test_pretty_join_two_items() {
        // Test with two items - should join with operator
        assert_eq!(pretty_join(&["a", "b"], "and"), "a and b");
        assert_eq!(pretty_join(&["x", "y"], "or"), "x or y");
    }

    #[test]
    fn test_pretty_join_multiple_items() {
        // Test with multiple items - comma separated with final operator
        assert_eq!(pretty_join(&["a", "b", "c"], "and"), "a, b and c");
        assert_eq!(pretty_join(&["x", "y", "z"], "or"), "x, y or z");
        assert_eq!(pretty_join(&[1, 2, 3, 4], "+"), "1, 2, 3 + 4");
    }

    #[test]
    fn test_pretty_join_empty() {
        // Test with empty array - should return empty string
        let empty: &[&str] = &[];
        assert_eq!(pretty_join(empty, "and"), "");
    }

    // ==================== is_all_caps tests ====================

    #[test]
    fn test_is_all_caps_valid() {
        // Test with valid all-caps names
        assert!(is_all_caps("HELLO"));
        assert!(is_all_caps("WORLD"));
        assert!(is_all_caps("TEST_CASE"));
        assert!(is_all_caps("XML"));
        assert!(is_all_caps("HTTP_RESPONSE"));
    }

    #[test]
    fn test_is_all_caps_invalid_lowercase() {
        // Test with lowercase - should return false
        assert!(!is_all_caps("hello"));
        assert!(!is_all_caps("world"));
        assert!(!is_all_caps("Hello"));
        assert!(!is_all_caps("Test"));
    }

    #[test]
    fn test_is_all_caps_with_numbers() {
        // Test with numbers - numbers are ignored in the check
        assert!(is_all_caps("TEST123"));
        assert!(is_all_caps("V2"));
        assert!(!is_all_caps("test123"));
        assert!(!is_all_caps("Test123"));
    }

    #[test]
    fn test_is_all_caps_empty() {
        // Test with empty string - filter returns empty, all() on empty is true
        // This is a quirk of the implementation
        assert!(is_all_caps(""));
    }

    #[test]
    fn test_is_all_caps_with_underscores() {
        // Test with underscores - underscores are allowed
        assert!(is_all_caps("CONSTANT_NAME"));
        assert!(is_all_caps("A_B_C"));
        assert!(!is_all_caps("constant_name"));
    }
}
