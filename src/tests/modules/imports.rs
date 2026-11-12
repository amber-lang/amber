//! Tests for modules/imports/import_string.rs

use crate::compiler::SourcePath;
use crate::modules::imports::import_string::ImportString;

#[cfg(test)]
mod import_string_tests {
    use super::*;

    #[test]
    fn test_import_string_new() {
        let imp = ImportString { value: String::new(), source_path: SourcePath::Stdin };
        assert!(imp.value.is_empty());
    }

    #[test]
    fn test_import_string_clone() {
        let imp = ImportString { value: "test.ab".to_string(), source_path: SourcePath::Stdin };
        let cloned = imp.clone();
        assert_eq!(imp.value, cloned.value);
    }
}
