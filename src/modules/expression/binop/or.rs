use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Or {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for Or {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Or {
    syntax_name!("Or");

    fn new() -> Self {
        Or {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
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
