use heraclitus_compiler::prelude::*;
use crate::{modules::types::Type, utils::ParserMetadata};
use super::expr::Expr;
use crate::modules::typecheck::TypeCheckModule;

pub mod is;
pub mod cast;


pub trait TypeOp: SyntaxModule<ParserMetadata> + TypeCheckModule {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Type);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}