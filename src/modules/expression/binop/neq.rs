use heraclitus_compiler::prelude::*;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type};
use crate::modules::{Type, Typed};

#[derive(Debug)]
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
        parse_left_expr(meta, &mut *self.left, "!=")?;
        let tok = meta.get_current_token();
        token(meta, "!=")?;
        syntax(meta, &mut *self.right)?;
        let error = "Cannot compare two values of different types";
        expression_arms_of_same_type(meta, &self.left, &self.right, tok, error);
        Ok(())
    }
}

impl TranslateModule for Neq {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        if self.left.get_type() == Type::Text && self.right.get_type() == Type::Text {
            format!("$([ \"_{left}\" == \"_{right}\" ]; echo $?)")
        } else {
            translate_computation(meta, ArithOp::Neq, Some(left), Some(right))
        }
    }
}