use super::BinOp;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Le {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Typed for Le {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Le {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "<=")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Le {
    syntax_name!("Le");

    fn new() -> Self {
        Le {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot compare two values of different types '{l_type}' <= '{r_type}'")
        };
        let comment = "You can only compare values of type 'Num'.";
        handle_binop!(meta, self.left, self.right, message, comment, [Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Le {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Le, Some(left), Some(right))
    }
}

impl DocumentationModule for Le {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
