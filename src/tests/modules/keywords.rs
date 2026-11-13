//! Tests for modules/keywords.rs

use crate::modules::keywords::{KeywordKind, KeywordRegistration, iter_keywords};

#[cfg(test)]
mod keyword_kind_tests {
    use super::*;

    #[test]
    fn test_keyword_kind_clone() {
        let kind = KeywordKind::Stmt;
        let cloned = kind;
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_keyword_kind_eq() {
        assert_eq!(KeywordKind::Stmt, KeywordKind::Stmt);
        assert_ne!(KeywordKind::Stmt, KeywordKind::BuiltinStmt);
        assert_eq!(KeywordKind::BinaryOp, KeywordKind::BinaryOp);
    }

    #[test]
    fn test_keyword_kind_debug() {
        let debug_str = format!("{:?}", KeywordKind::BuiltinExpr);
        assert_eq!(debug_str, "BuiltinExpr");
    }
}

#[cfg(test)]
mod keyword_registration_tests {
    use super::*;

    #[test]
    fn test_keyword_registration_new_with_kind() {
        let reg = KeywordRegistration::new_with_kind(
            "TestKeyword",
            "test",
            KeywordKind::BuiltinStmt,
        );
        assert_eq!(reg.struct_name, "TestKeyword");
        assert_eq!(reg.keyword, "test");
        assert_eq!(reg.kind, KeywordKind::BuiltinStmt);
    }

    #[test]
    fn test_keyword_registration_new_default() {
        let reg = KeywordRegistration::new("MyKeyword", "mykeyword");
        assert_eq!(reg.struct_name, "MyKeyword");
        assert_eq!(reg.keyword, "mykeyword");
        assert_eq!(reg.kind, KeywordKind::Stmt); // Default for new()
    }

    #[test]
    fn test_keyword_registration_clone() {
        let reg = KeywordRegistration::new("Test", "test");
        let cloned = reg.clone();
        assert_eq!(reg, cloned);
    }

    #[test]
    fn test_keyword_registration_eq() {
        let reg1 = KeywordRegistration::new("Test", "test");
        let reg2 = KeywordRegistration::new("Test", "test");
        let reg3 = KeywordRegistration::new("Other", "other");
        
        assert_eq!(reg1, reg2);
        assert_ne!(reg1, reg3);
    }
}

#[cfg(test)]
mod iter_keywords_tests {
    use super::*;

    #[test]
    fn test_iter_keywords_returns_iterator() {
        let keywords: Vec<_> = iter_keywords().collect();
        // Should have at least some keywords registered
        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_iter_keywords_has_variety() {
        let keywords: Vec<_> = iter_keywords().collect();
        
        let mut has_stmt = false;
        let mut has_builtin = false;
        let mut has_binary_op = false;
        
        for kw in &keywords {
            match kw.kind {
                KeywordKind::Stmt => has_stmt = true,
                KeywordKind::BuiltinStmt | KeywordKind::BuiltinExpr => has_builtin = true,
                KeywordKind::BinaryOp => has_binary_op = true,
            }
        }
        
        // Should have variety of keyword kinds
        let variety_count = (has_stmt as u8) + (has_builtin as u8) + (has_binary_op as u8);
        assert!(variety_count >= 2, "Expected at least 2 different keyword kinds, found {}", variety_count);
    }

}

#[cfg(test)]
mod reserved_identifier_tests {
    use super::*;
    use crate::modules::variable::variable_name_keywords;

    #[test]
    fn test_registered_keywords_are_reserved_identifiers() {
        let reserved = variable_name_keywords();
        let missing: Vec<_> = iter_keywords()
            .filter(|kw| kw.kind != KeywordKind::BinaryOp)
            .map(|kw| kw.keyword)
            .filter(|kw| !reserved.contains(kw))
            .collect();
        assert!(
            missing.is_empty(),
            "Keywords missing from variable_name_keywords(): {missing:?}"
        );
    }
}
