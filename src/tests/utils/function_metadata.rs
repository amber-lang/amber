//! Tests for utils/function_metadata.rs

use crate::modules::types::Type;
use crate::utils::function_metadata::FunctionMetadata;

#[cfg(test)]
mod function_metadata_tests {
    use super::*;

    #[test]
    fn test_function_metadata_new() {
        let _meta = FunctionMetadata::new("test_func", 1, 0, &Type::Int);
    }

    #[test]
    fn test_function_metadata_mangled_name_uppercase() {
        let meta = FunctionMetadata::new("CONSTANT", 42, 3, &Type::Int);
        assert_eq!(meta.mangled_name(), "__ret_CONSTANT42_v3");
    }

    #[test]
    fn test_function_metadata_mangled_name_lowercase() {
        let meta = FunctionMetadata::new("my_function", 1, 0, &Type::Text);
        assert_eq!(meta.mangled_name(), "ret_my_function1_v0");
    }

    #[test]
    fn test_function_metadata_get_type() {
        let meta = FunctionMetadata::new("func", 1, 0, &Type::Bool);
        assert_eq!(meta.get_type(), Type::Bool);
    }

    #[test]
    fn test_mangle_with_underscore() {
        let meta = FunctionMetadata::new("MY_CONSTANT", 5, 2, &Type::Num);
        assert_eq!(meta.mangled_name(), "__ret_MY_CONSTANT5_v2");
    }

    #[test]
    fn test_mangle_with_numbers() {
        let meta = FunctionMetadata::new("func123", 1, 0, &Type::Int);
        assert_eq!(meta.mangled_name(), "ret_func1231_v0");
    }
}
