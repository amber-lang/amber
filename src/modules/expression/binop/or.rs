use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

use super::BinOp;

#[derive(Debug, Clone)]
pub struct Or {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Typed for Or {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Or {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "or")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Or {
    syntax_name!("Or");

    fn new() -> Self {
        Or {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let error = "Logical 'or' operation can only be used on arguments of the same type";
        handle_binop!(meta, self.left, self.right, error)?;
        Ok(())
    }
}

impl TranslateModule for Or {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Or, Some(left), Some(right))
    }
}

impl DocumentationModule for Or {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
