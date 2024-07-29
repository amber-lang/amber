use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, handle_binop, utils::TranslateMetadata};
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::translate::compute::ArithOp;
use crate::utils::metadata::ParserMetadata;
use crate::translate::compute::translate_computation;
use crate::modules::types::{Typed, Type};
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone)]
pub struct Div {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for Div {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Div {
    syntax_name!("Div");

    fn new() -> Self {
        Div {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot divide value of type '{l_type}' with value of type '{r_type}'")
        };
        let comment = "You can only divide values of type 'Num'.";
        handle_binop!(meta, self.left, self.right, message, comment, [Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Div {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Div, Some(left), Some(right))
    }
}

impl DocumentationModule for Div {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
