use heraclitus_compiler::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::metadata::ParserMetadata;
use crate::utils::pluralize;
use super::super::expression::expr::Expr;

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

pub trait BinOp: SyntaxModule<ParserMetadata> {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;

    fn typecheck_allowed_types(
        meta: &mut ParserMetadata,
        operator: &str,
        left: &Expr,
        right: &Expr,
        allowed_types: &[Type],
    ) -> Result<Type, Failure> {
        let left_type = left.get_type();
        let right_type = right.get_type();
        let left_match = allowed_types.iter().any(|types| left_type.is_allowed_in(types));
        let right_match = allowed_types.iter().any(|types| right_type.is_allowed_in(types));
        if !left_match || !right_match {
            let pretty_types = Type::pretty_join(allowed_types, "and");
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
        left: &Expr,
        right: &Expr,
    ) -> Result<Type, Failure> {
        let left_type = left.get_type();
        let right_type = right.get_type();
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

pub fn get_binop_position_info(meta: &ParserMetadata, left: &Expr, right: &Expr) -> PositionInfo {
    let begin = meta.get_token_at(left.pos.0);
    let end = meta.get_token_at(right.pos.1);
    PositionInfo::from_between_tokens(meta, begin, end)
}
