use heraclitus_compiler::prelude::*;
use crate::{modules::types::Type, utils::ParserMetadata};
use super::expr::Expr;

pub mod is;
pub mod cast;


pub trait TypeOp: SyntaxModule<ParserMetadata> {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Type);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}