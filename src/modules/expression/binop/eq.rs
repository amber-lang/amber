use super::{strip_text_quotes, BinOp};
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Eq {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Typed for Eq {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Eq {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "==")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Eq {
    syntax_name!("Eq");

    fn new() -> Self {
        Eq {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot compare two values of different types '{l_type}' == '{r_type}'")
        };
        handle_binop!(meta, self.left, self.right, message)?;
        Ok(())
    }
}

impl TranslateModule for Eq {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let mut left = self.left.translate(meta);
        let mut right = self.right.translate(meta);
        // Handle text comparison
        if self.left.get_type() == Type::Text && self.right.get_type() == Type::Text {
            strip_text_quotes(&mut left);
            strip_text_quotes(&mut right);
            meta.gen_subprocess(&format!("[ \"_{left}\" != \"_{right}\" ]; echo $?"))
        } else {
            translate_computation(meta, ArithOp::Eq, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Eq {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
