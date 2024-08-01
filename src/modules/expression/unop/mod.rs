use super::expr::Expr;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

pub mod neg;
pub mod not;

pub trait UnOp: SyntaxModule<ParserMetadata> {
    fn set_expr(&mut self, expr: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}
