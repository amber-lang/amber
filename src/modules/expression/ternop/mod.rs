use super::expr::Expr;
use crate::modules::typecheck::TypeCheckModule;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

pub mod ternary;

pub trait TernOp: SyntaxModule<ParserMetadata> + TypeCheckModule {
    fn set_left(&mut self, left: Expr);
    fn set_middle(&mut self, middle: Expr);
    fn set_right(&mut self, right: Expr);
    fn parse_operator_left(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
    fn parse_operator_right(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}
