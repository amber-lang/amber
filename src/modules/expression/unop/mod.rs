use heraclitus_compiler::prelude::*;
use crate::utils::ParserMetadata;
use super::expr::Expr;

pub mod not;
pub mod neg;

pub trait UnOp: SyntaxModule<ParserMetadata> {
    fn set_expr(&mut self, expr: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}