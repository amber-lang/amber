use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::strip_text_quotes;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Neq {
    left: Box<Expr>,
    right: Box<Expr>
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
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "!=")?;
        let tok = meta.get_current_token();
        token(meta, "!=")?;
        syntax(meta, &mut *self.right)?;
        let l_type = self.left.get_type();
        let r_type = self.right.get_type();
        let message = format!("Cannot compare two values of different types '{l_type}' != '{r_type}'");
        expression_arms_of_same_type(meta, &self.left, &self.right, tok, &message)?;
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
    fn document(&self) -> String {
        "".to_string()
    }
}
