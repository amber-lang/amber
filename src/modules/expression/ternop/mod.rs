use heraclitus_compiler::prelude::*;
use crate::utils::ParserMetadata;
use super::expr::Expr;

pub mod ternary;

pub trait TernOp: SyntaxModule<ParserMetadata> {
    fn set_left(&mut self, left: Expr);
    fn set_middle(&mut self, middle: Expr);
    fn set_right(&mut self, right: Expr);
    fn parse_operator_left(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
    fn parse_operator_right(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}