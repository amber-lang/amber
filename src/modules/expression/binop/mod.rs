use heraclitus_compiler::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::metadata::ParserMetadata;
use crate::utils::pluralize;
use super::super::expression::expr::{Expr, ExprType};
use crate::modules::typecheck::TypeCheckModule;
use crate::utils::pretty_join;

pub mod add;
pub mod sub;
pub mod mul;
pub mod div;
pub mod modulo;
pub mod and;
pub mod or;
pub mod gt;
pub mod ge;
pub mod lt;
pub mod le;
pub mod eq;
pub mod neq;
pub mod range;

pub trait BinOp: SyntaxModule<ParserMetadata> + TypeCheckModule {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;

    fn typecheck_allowed_types(
        meta: &mut ParserMetadata,
        operator: &str,
        left: &mut Expr,
        right: &mut Expr,
        allowed_types: &[Type],
    ) -> Result<Type, Failure> {
        let left_type = left.get_type();
        let right_type = right.get_type();
        let left_match = allowed_types.iter().any(|types| left_type.is_allowed_in(types));
        let right_match = allowed_types.iter().any(|types| right_type.is_allowed_in(types));
        if !left_match || !right_match {
            let pretty_types = pretty_join(allowed_types, "and");
            let comment = pluralize(allowed_types.len(), "Allowed type is", "Allowed types are");
            let pos = get_binop_position_info(meta, left, right);
            let message = Message::new_err_at_position(meta, pos)
                .message(format!("Cannot perform {operator} on value of type '{left_type}' and value of type '{right_type}'"))
                .comment(format!("{comment} {pretty_types}."));
            Err(Failure::Loud(message))
        } else {
            Self::typecheck_equality(meta, left, right)
        }
    }

    fn typecheck_equality(
        meta: &mut ParserMetadata,
        left: &mut Expr,
        right: &mut Expr,
    ) -> Result<Type, Failure> {
        match (left.get_type(), right.get_type()) {
            (Type::Int, Type::Num) | (Type::Num, Type::Int) => {
                Ok(Type::Num)
            }
            // Array type inference
            (Type::Array(left_inner), Type::Array(right_inner)) if *left_inner == Type::Generic || *right_inner == Type::Generic => {
                if *left_inner == Type::Generic && *right_inner != Type::Generic {
                    if let Some(ExprType::VariableGet(var)) = &left.value {
                        meta.update_var_type(&var.name, Type::Array(right_inner.clone()));
                    }
                    left.kind = Type::Array(right_inner.clone());
                    Ok(Type::Array(right_inner))
                } else if *left_inner != Type::Generic && *right_inner == Type::Generic {
                    if let Some(ExprType::VariableGet(var)) = &right.value {
                        meta.update_var_type(&var.name, Type::Array(left_inner.clone()));
                    }
                    right.kind = Type::Array(left_inner.clone());
                    Ok(Type::Array(left_inner))
                } else {
                    Ok(Type::Array(Box::new(Type::Generic)))
                }
            }
            (left_type, right_type) => {
                if left_type != right_type {
                    let pos = get_binop_position_info(meta, left, right);
                    let message = Message::new_err_at_position(meta, pos)
                        .message(format!("Expected both operands to be of the same type, but got '{left_type}' and '{right_type}'."));
                    Err(Failure::Loud(message))
                } else {
                    Ok(left_type)
                }
            }
        }
    }
}

pub fn get_binop_position_info(meta: &mut ParserMetadata, left: &Expr, right: &Expr) -> PositionInfo {
    let left_pos = left.get_position();
    let right_pos = right.get_position();
    PositionInfo::from_between_positions(meta, left_pos, right_pos)
}
