use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};


#[derive(Debug, Clone)]
pub struct And {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for And {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for And {
    syntax_name!("And");

    fn new() -> Self {
        And {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let error = "Logical 'and' operation can only be used on arguments of the same type";
        handle_binop!(meta, self.left, self.right, error)?;
        Ok(())
    }
}
impl TranslateModule for And {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::And, Some(left), Some(right))
    }
}

impl DocumentationModule for And {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
