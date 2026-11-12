//! Tests for utils/context.rs, function_cache.rs, function_interface.rs

use crate::compiler::SourcePath;
use crate::modules::function::core::signature::{FunctionDeclId, FunctionSignature};
use crate::modules::types::Type;
use crate::utils::context::{Context, ScopeUnit, VariableDecl};

#[cfg(test)]
mod context_tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let path = SourcePath::StdLib("std/test".to_string());
        let ctx = Context::new(path.clone(), vec![]);
        assert_eq!(ctx.index, 0);
        assert_eq!(ctx.path, path);
        assert!(ctx.scopes.is_empty());
        assert!(!ctx.is_fun_ctx);
        assert!(!ctx.is_loop_ctx);
    }

    #[test]
    fn test_scope_unit_add_var() {
        let mut scope = ScopeUnit::new();
        let var = VariableDecl::new("x".to_string(), Type::Int);

        // First add should succeed
        assert!(scope.add_var(var));

        // Adding same name should fail (returns false)
        let var2 = VariableDecl::new("x".to_string(), Type::Text);
        assert!(!scope.add_var(var2));
    }

    #[test]
    fn test_scope_unit_get_var() {
        let mut scope = ScopeUnit::new();
        let var = VariableDecl::new("x".to_string(), Type::Int);
        scope.add_var(var);

        assert!(scope.get_var("x").is_some());
        assert!(scope.get_var("y").is_none());
    }

    #[test]
    fn test_scope_unit_add_fun() {
        let mut scope = ScopeUnit::new();
        let fun = FunctionSignature {
            name: "foo".to_string(),
            params: vec![],
            returns: Type::Null,
            is_public: false,
            is_failable: false,
            id: FunctionDeclId::new(1),
        };

        assert!(scope.add_fun(fun));
        assert!(!scope.add_fun(FunctionSignature {
            name: "foo".to_string(),
            params: vec![],
            returns: Type::Null,
            is_public: false,
            is_failable: false,
            id: FunctionDeclId::new(2),
        }));
    }

    #[test]
    fn test_scope_unit_add_fun_as() {
        let mut scope = ScopeUnit::new();
        let fun = FunctionSignature {
            name: "foo".to_string(),
            params: vec![],
            returns: Type::Null,
            is_public: false,
            is_failable: false,
            id: FunctionDeclId::new(1),
        };

        assert!(scope.add_fun_as("bar".to_string(), fun.clone()));
        // The function is visible under the alias only, but keeps its declared name
        assert_eq!(scope.get_fun("bar").map(|fun| fun.name.as_str()), Some("foo"));
        assert!(scope.get_fun("foo").is_none());
        // Both names can coexist in the same scope
        assert!(scope.add_fun(fun));
        assert!(!scope.add_fun_as("bar".to_string(), scope.get_fun("foo").unwrap().clone()));
    }

    #[test]
    fn test_scope_unit_add_var_as() {
        let mut scope = ScopeUnit::new();
        let var = VariableDecl::new("foo".to_string(), Type::Int);

        assert!(scope.add_var_as("bar".to_string(), var.clone()));
        // The variable is visible under the alias only, but keeps its declared name
        assert_eq!(scope.get_var("bar").map(|var| var.name.as_str()), Some("foo"));
        assert!(scope.get_var("foo").is_none());
        // Both names can coexist in the same scope
        assert!(scope.add_var(var.clone()));
        assert!(!scope.add_var_as("bar".to_string(), var));
    }

    #[test]
    fn test_variable_decl_new() {
        let var = VariableDecl::new("test".to_string(), Type::Text);
        assert_eq!(var.name, "test");
        assert_eq!(var.kind, Type::Text);
        assert!(!var.is_const);
        assert!(!var.is_ref);
        assert!(!var.is_used);
        assert!(!var.is_modified);
    }

    #[test]
    fn test_variable_decl_with_methods() {
        let var = VariableDecl::new("x".to_string(), Type::Int)
            .with_const(true)
            .with_ref(true)
            .with_public(true);

        assert!(var.is_const);
        assert!(var.is_ref);
        assert!(var.is_public);
    }
}
