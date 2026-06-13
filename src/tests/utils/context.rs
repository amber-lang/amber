//! Tests for utils/context.rs, function_cache.rs, function_interface.rs

use crate::utils::context::{Context, ScopeUnit, VariableDecl, FunctionDecl};
use crate::modules::types::Type;

#[cfg(test)]
mod context_tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = Context::new(Some("test.ab".to_string()), vec![]);
        assert_eq!(ctx.index, 0);
        assert_eq!(ctx.path, Some("test.ab".to_string()));
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
        let fun = FunctionDecl {
            name: "foo".to_string(),
            args: vec![],
            returns: Type::Null,
            is_args_typed: true,
            is_public: false,
            is_failable: false,
            id: 1,
        };
        
        assert!(scope.add_fun(fun));
        assert!(!scope.add_fun(FunctionDecl {
            name: "foo".to_string(),
            args: vec![],
            returns: Type::Null,
            is_args_typed: true,
            is_public: false,
            is_failable: false,
            id: 2,
        }));
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


