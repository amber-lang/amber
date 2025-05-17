use heraclitus_compiler::prelude::*;
use crate::{modules::types::{Type, Typed}, utils::ParserMetadata};
use super::expr::Expr;

pub mod not;
pub mod neg;

pub trait UnOp: SyntaxModule<ParserMetadata> {
    fn set_expr(&mut self, expr: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;

    fn typecheck_allowed_types(
        meta: &mut ParserMetadata,
        operator: &str,
        expr: &Expr,
        allowed_types: &[Type],
    ) -> Result<Type, Failure> {
        let expr_type = expr.get_type();
        let expr_match = allowed_types.iter().any(|types| expr_type.is_allowed_in(types));
        if !expr_match {
            let message = expr.get_error_message(meta);
            let msg = format!("Cannot {operator} value of type '{expr_type}'");
            let pretty_types = Type::pretty_disjunction(allowed_types);
            let comment = format!("You can only {operator} values of type {pretty_types} together.");
            Err(Failure::Loud((message).message(msg).comment(comment)))
        } else {
            Ok(expr_type)
        }
    }
}
