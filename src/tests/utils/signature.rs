use crate::modules::expression::expr::Expr;
use crate::modules::function::core::signature::{FunctionDeclId, FunctionParam, FunctionSignature};
use crate::modules::types::Type;
use heraclitus_compiler::prelude::SyntaxModule;

#[cfg(test)]
mod signature_tests {
    use super::*;

    fn signature(params: Vec<FunctionParam>) -> FunctionSignature {
        FunctionSignature {
            id: FunctionDeclId::new(1),
            name: "test_func".to_string(),
            params,
            returns: Type::Null,
            is_public: true,
            is_failable: false,
        }
    }

    #[test]
    fn test_is_fully_typed() {
        let typed = signature(vec![FunctionParam::new("x".to_string(), Type::Int)]);
        assert!(typed.is_fully_typed());
        assert!(!typed.has_mixed_typing());

        let generic = signature(vec![FunctionParam::new("x".to_string(), Type::Generic)]);
        assert!(!generic.is_fully_typed());
        assert!(!generic.has_mixed_typing());
    }

    #[test]
    fn test_no_params_is_fully_typed() {
        assert!(signature(vec![]).is_fully_typed());
        assert!(!signature(vec![]).has_mixed_typing());
    }

    #[test]
    fn test_mixed_typing_is_detected() {
        let mixed = signature(vec![
            FunctionParam::new("x".to_string(), Type::Int),
            FunctionParam::new("y".to_string(), Type::Generic),
        ]);
        assert!(mixed.has_mixed_typing());
        assert!(!mixed.is_fully_typed());
    }

    #[test]
    fn test_arity_counts_defaults_separately() {
        let sig = signature(vec![
            FunctionParam::new("x".to_string(), Type::Int),
            FunctionParam::new("y".to_string(), Type::Int).with_default(Some(Expr::new())),
        ]);
        assert_eq!(sig.total_arity(), 2);
        assert_eq!(sig.required_arity(), 1);
        assert_eq!(sig.optional_arity(), 1);
    }

    #[test]
    fn test_param_types_preserve_order() {
        let sig = signature(vec![
            FunctionParam::new("x".to_string(), Type::Int),
            FunctionParam::new("y".to_string(), Type::Text),
        ]);
        assert_eq!(sig.param_types(), vec![Type::Int, Type::Text]);
    }

    #[test]
    fn test_defaults_after() {
        let sig = signature(vec![
            FunctionParam::new("a".to_string(), Type::Int),
            FunctionParam::new("b".to_string(), Type::Int).with_default(Some(Expr::new())),
            FunctionParam::new("c".to_string(), Type::Int).with_default(Some(Expr::new())),
        ]);
        assert_eq!(sig.defaults_after(1).count(), 2);
        assert_eq!(sig.defaults_after(2).count(), 1);
        assert_eq!(sig.defaults_after(3).count(), 0);
    }
}
