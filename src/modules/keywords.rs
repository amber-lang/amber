// Module for keyword extraction from Amber Rust code.
// This module defines traits that allow importing Rust modules directly
// to extract keywords without regex parsing.

#![allow(dead_code)]

/// The kind of keyword (statement, builtin, or binary operation)
/// Fixed set of variants: Stmt, BuiltinStmt, BuiltinExpr, BinaryOp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
    /// Statement keyword (if, while, for, etc.)
    Stmt,
    /// Builtin statement function (echo, cd, clear, etc.)
    BuiltinStmt,
    /// Builtin expression function (len, lines, ls, etc.)
    BuiltinExpr,
    /// Binary operator keyword (and, or, add, sub, etc.)
    BinaryOp,
}

/// A registered keyword for auto-discovery via inventory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordRegistration {
    /// The name of the type implementing the keyword trait
    pub struct_name: &'static str,
    /// The keyword string
    pub keyword: &'static str,
    /// The kind of keyword
    pub kind: KeywordKind,
}

impl KeywordRegistration {
    /// Create a new keyword registration entry
    pub const fn new_with_kind(
        struct_name: &'static str,
        keyword: &'static str,
        kind: KeywordKind,
    ) -> Self {
        Self {
            struct_name,
            keyword,
            kind,
        }
    }

    #[allow(dead_code)]
    pub const fn new(struct_name: &'static str, keyword: &'static str) -> Self {
        Self {
            struct_name,
            keyword,
            kind: KeywordKind::Stmt,
        }
    }
}

// Declare the registry using collect! macro
inventory::collect!(KeywordRegistration);

/// Public function to iterate over registered keywords
pub fn iter_keywords() -> impl Iterator<Item = &'static KeywordRegistration> {
    inventory::iter::<KeywordRegistration>()
}

/// Trait for statement keywords (if, while, for, loop, break, continue, return, etc.)
#[allow(dead_code)]
pub trait KeywordStmt {
    fn keyword_stmt() -> &'static str;
}

/// Trait for expression keywords (and, or, not)
#[allow(dead_code)]
pub trait KeywordExpr {
    fn keyword_expr() -> &'static str;
}

/// Trait for builtin functions (cd, echo, len, lines, etc.)
#[allow(dead_code)]
pub trait BuiltinName {
    fn builtin_name() -> &'static str;
}
