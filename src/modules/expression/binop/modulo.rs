use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::translate::compute::{ArithOp, translate_computation};
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Modulo {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for Modulo {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Modulo {
    syntax_name!("Modulo");

    fn new() -> Self {
        Modulo {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot apply modulo on value of type '{l_type}' with value of type '{r_type}'")
        };
        let comment = "You can only apply modulo on values of type 'Num'.";
        handle_binop!(meta, self.left, self.right, message, comment, [Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Modulo {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Modulo, Some(left), Some(right))
    }
}

impl DocumentationModule for Modulo {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
