use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Mul {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>
}

impl Typed for Mul {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Mul {
    syntax_name!("Mul");

    fn new() -> Self {
        Mul {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot multiply value of type '{l_type}' with value of type '{r_type}'")
        };
        let comment = "You can only multiply values of type 'Num'.";
        handle_binop!(meta, self.left, self.right, message, comment, [Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Mul {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Mul, Some(left), Some(right))
    }
}

impl DocumentationModule for Mul {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
