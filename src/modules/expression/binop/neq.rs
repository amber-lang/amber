use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::strip_text_quotes;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Neq {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for Neq {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Neq {
    syntax_name!("Neq");

    fn new() -> Self {
        Neq {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot compare two values of different types '{l_type}' != '{r_type}'")
        };
        handle_binop!(meta, self.left, self.right, message)?;
        Ok(())
    }
}

impl TranslateModule for Neq {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let mut left = self.left.translate(meta);
        let mut right = self.right.translate(meta);
        if self.left.get_type() == Type::Text && self.right.get_type() == Type::Text {
            strip_text_quotes(&mut left);
            strip_text_quotes(&mut right);
            meta.gen_subprocess(&format!("[ \"_{left}\" == \"_{right}\" ]; echo $?"))
        } else {
            translate_computation(meta, ArithOp::Neq, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Neq {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
