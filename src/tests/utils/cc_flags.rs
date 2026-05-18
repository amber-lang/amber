//! Tests for utils/cc_flags.rs

use crate::utils::cc_flags::{CCFlags, get_ccflag_by_name, get_ccflag_name};

#[cfg(test)]
mod cc_flags_tests {
    use super::*;

    #[test]
    fn test_get_ccflag_by_name_valid() {
        assert_eq!(get_ccflag_by_name("allow_nested_if_else"), CCFlags::AllowNestedIfElse);
        assert_eq!(get_ccflag_by_name("allow_camel_case"), CCFlags::AllowCamelCase);
        assert_eq!(get_ccflag_by_name("allow_absurd_cast"), CCFlags::AllowAbsurdCast);
        assert_eq!(get_ccflag_by_name("allow_dead_code"), CCFlags::AllowDeadCode);
        assert_eq!(get_ccflag_by_name("allow_public_mutable"), CCFlags::AllowPublicMutable);
    }

    #[test]
    fn test_get_ccflag_by_name_invalid() {
        assert_eq!(get_ccflag_by_name("invalid_flag"), CCFlags::UndefinedFlag);
        assert_eq!(get_ccflag_by_name(""), CCFlags::UndefinedFlag);
    }

    #[test]
    fn test_get_ccflag_name() {
        assert_eq!(get_ccflag_name(CCFlags::AllowNestedIfElse), "allow_nested_if_else");
        assert_eq!(get_ccflag_name(CCFlags::AllowCamelCase), "allow_camel_case");
        assert_eq!(get_ccflag_name(CCFlags::UndefinedFlag), "undefined_flag");
    }

    #[test]
    fn test_ccflags_clone() {
        let flag = CCFlags::AllowDeadCode;
        let cloned = flag;
        assert_eq!(flag, cloned);
    }
}
