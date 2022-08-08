use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type, expression_arms_of_type};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Div {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Div {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Div {
    syntax_name!("Div");

    fn new() -> Self {
        Div {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Void
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.left, "/")?;
        let tok = meta.get_current_token();
        token(meta, "/")?;
        syntax(meta, &mut *self.right)?;
        let error = "Divide operation can only divide numbers";
        expression_arms_of_type(meta, &self.left, &self.right, Type::Num, tok, error);
        Ok(())
    }
}