use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Gt {
    left: Box<Expr>,
    right: Box<Expr>
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
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.left, ">")?;
        let tok = meta.get_current_token();
        token(meta, ">")?;
        syntax(meta, &mut *self.right)?;
        let error = "Cannot compare two values of different types";
        expression_arms_of_same_type(meta, &self.left, &self.right, tok, error);
        Ok(())
    }
}