use heraclitus_compiler::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::metadata::ParserMetadata;
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
        if !left_match || !right_match || left_type != right_type {
            let pos = get_binop_position_info(meta, left, right);
            let message = Message::new_err_at_position(meta, pos);
            let msg = format!("Cannot {operator} value of type '{left_type}' with value of type '{right_type}'");
            let pretty_types = Type::pretty_disjunction(allowed_types);
            let comment = format!("You can only {operator} values of type {pretty_types} together.");
            Err(Failure::Loud(message.message(msg).comment(comment)))
        } else {
            Ok(left_type)
        }
    }

    fn typecheck_equality(
        meta: &mut ParserMetadata,
        operator: &str,
        left: &Expr,
        right: &Expr,
    ) -> Result<Type, Failure> {
        let left_type = left.get_type();
        let right_type = right.get_type();
        if left_type != right_type {
            let pos = get_binop_position_info(meta, left, right);
            let message = Message::new_err_at_position(meta, pos);
            let msg = format!("Cannot {operator} value of type '{left_type}' with value of type '{right_type}'");
            let comment = format!("You can only {operator} values of the same types.");
            Err(Failure::Loud(message.message(msg).comment(comment)))
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
