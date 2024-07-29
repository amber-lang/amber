use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::translate::compute::{ArithOp, translate_computation};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Gt {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for Gt {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Gt {
    syntax_name!("Gt");

    fn new() -> Self {
        Gt {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot compare two values of different types '{l_type}' > '{r_type}'")
        };
        let comment = "You can only compare values of type 'Num'.";
        handle_binop!(meta, self.left, self.right, message, comment, [Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Gt {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Gt, Some(left), Some(right))
    }
}

impl DocumentationModule for Gt {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
