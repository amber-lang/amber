use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, TranslateMetadata}, translate::compute::{translate_computation, ArithOp}};
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::{Type, Typed};
use crate::translate::module::TranslateModule;

#[derive(Debug)]
pub struct Div {
    left: Box<Expr>,
    right: Box<Expr>
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
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "/")?;
        let tok = meta.get_current_token();
        token(meta, "/")?;
        syntax(meta, &mut *self.right)?;
        let error = "Divide operation can only divide numbers";
        let l_type = self.left.get_type();
        let r_type = self.right.get_type();
        expression_arms_of_type(meta, &l_type, &r_type, &[Type::Num], tok, error);
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