use heraclitus_compiler::prelude::*;
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use crate::utils::pluralize;

/// Context for type checking operations
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Reference to parser metadata for error reporting and context
    pub metadata: *mut ParserMetadata,
}

impl TypeContext {
    pub fn new(metadata: &mut ParserMetadata) -> Self {
        Self {
            metadata: metadata as *mut ParserMetadata,
        }
    }

    /// Get a reference to the parser metadata
    pub fn metadata(&self) -> &ParserMetadata {
        unsafe { &*self.metadata }
    }

    /// Get a mutable reference to the parser metadata
    pub fn metadata_mut(&mut self) -> &mut ParserMetadata {
        unsafe { &mut *self.metadata }
    }
}

/// Result type for type checking operations
pub type TypeCheckResult<T> = Result<T, Failure>;

/// AST node that has been type checked
#[derive(Debug, Clone)]
pub struct TypedAstNode {
    pub kind: Type,
    // Additional type checking metadata can be added here
}

/// Trait for modules that can perform type checking
pub trait TypeCheckModule {
    /// Perform type checking on this module
    fn type_check(&mut self, ctx: &mut TypeContext) -> TypeCheckResult<Type>;
}

/// Utility function for type checking binary operations
pub fn typecheck_binary_allowed_types(
    meta: &mut ParserMetadata,
    operator: &str,
    left_type: &Type,
    right_type: &Type,
    allowed_types: &[Type],
    pos: PositionInfo,
) -> TypeCheckResult<Type> {
    let left_match = allowed_types.iter().any(|types| left_type.is_allowed_in(types));
    let right_match = allowed_types.iter().any(|types| right_type.is_allowed_in(types));
    if !left_match || !right_match {
        let pretty_types = Type::pretty_join(allowed_types, "and");
        let comment = pluralize(allowed_types.len(), "Allowed type is", "Allowed types are");
        let message = Message::new_err_at_position(meta, pos)
            .message(format!("Cannot perform {operator} on value of type '{left_type}' and value of type '{right_type}'"))
            .comment(format!("{comment} {pretty_types}."));
        Err(Failure::Loud(message))
    } else {
        typecheck_binary_equality(meta, left_type, right_type, pos)
    }
}

/// Utility function for type equality checking in binary operations
pub fn typecheck_binary_equality(
    meta: &mut ParserMetadata,
    left_type: &Type,
    right_type: &Type,
    pos: PositionInfo,
) -> TypeCheckResult<Type> {
    match (left_type, right_type) {
        (Type::Int, Type::Num) | (Type::Num, Type::Int) => {
            Ok(Type::Num)
        }
        (left_type, right_type) => {
            if left_type != right_type {
                let message = Message::new_err_at_position(meta, pos)
                    .message(format!("Expected both operands to be of the same type, but got '{left_type}' and '{right_type}'."));
                Err(Failure::Loud(message))
            } else {
                Ok(left_type.clone())
            }
        }
    }
}