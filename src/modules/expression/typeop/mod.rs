use super::expr::Expr;
use crate::modules::typecheck::TypeCheckModule;
use crate::{modules::types::Type, utils::ParserMetadata};
use heraclitus_compiler::prelude::*;

pub mod cast;
pub mod is;

pub trait TypeOp: SyntaxModule<ParserMetadata> + TypeCheckModule {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Type);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}
