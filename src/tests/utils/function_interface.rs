use crate::modules::function::declaration::FunctionDeclarationArgument;
use crate::modules::types::Type;
use crate::utils::function_interface::FunctionInterface;
#[cfg(test)]
mod function_interface_tests {
    use super::*;

    #[test]
    fn test_function_interface_into_fun_declaration() {
        let interface = FunctionInterface {
            id: Some(1),
            name: "test_func".to_string(),
            args: vec![
                FunctionDeclarationArgument {
                    name: "x".to_string(),
                    kind: Type::Int,
                    optional: None,
                    is_ref: false,
                    tok: None,
                }
            ],
            returns: Type::Null,
            is_public: true,
            is_failable: false,
        };

        let decl = interface.into_fun_declaration(1);
        assert_eq!(decl.name, "test_func");
        assert_eq!(decl.args.len(), 1);
        assert_eq!(decl.returns, Type::Null);
        assert!(decl.is_public);
        assert!(decl.is_args_typed);
    }

    #[test]
    fn test_function_interface_untyped_args() {
        let interface = FunctionInterface {
            id: Some(1),
            name: "generic_func".to_string(),
            args: vec![
                FunctionDeclarationArgument {
                    name: "x".to_string(),
                    kind: Type::Generic,
                    optional: None,
                    is_ref: false,
                    tok: None,
                }
            ],
            returns: Type::Null,
            is_public: false,
            is_failable: true,
        };

        let decl = interface.into_fun_declaration(1);
        assert!(!decl.is_args_typed);
        assert!(decl.is_failable);
    }
}
