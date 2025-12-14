use heraclitus_compiler::prelude::*;
use crate::{modules::types::{Type, Typed}, utils::{pluralize, ParserMetadata}};
use super::expr::Expr;
use crate::modules::typecheck::TypeCheckModule;
use crate::utils::pretty_join;

pub mod not;
pub mod neg;

pub trait UnOp: SyntaxModule<ParserMetadata> + TypeCheckModule {
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
            let msg = format!("Cannot perform {operator} on value of type '{expr_type}'");
            let pretty_types = pretty_join(allowed_types, "and");
            let sentence = pluralize(allowed_types.len(), "Allowed type is", "Allowed types are");
            let comment = format!("{sentence} {pretty_types}.");
            Err(Failure::Loud((message).message(msg).comment(comment)))
        } else {
            Ok(expr_type)
        }
    }
}
