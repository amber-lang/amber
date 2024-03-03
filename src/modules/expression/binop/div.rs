use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, translate::compute::{translate_computation, ArithOp}, utils::{metadata::ParserMetadata, TranslateMetadata}};
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::types::{Typed, Type};
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone)]
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
        let predicate = |kind| matches!(kind, Type::Num);
        expression_arms_of_type(meta, &l_type, &r_type, predicate, tok, error)?;
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
    fn document(&self) -> String {
        "".to_string()
    }
}
