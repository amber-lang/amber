use heraclitus_compiler::prelude::*;
use crate::utils::{metadata::ParserMetadata, error::get_error_logger};
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Add {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Add {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Add {
    syntax_name!("Add");

    fn new() -> Self {
        Add {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Num
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.left, "+")?;
        let tok = meta.get_current_token();
        token(meta, "+")?;
        syntax(meta, &mut *self.right)?;
        // If left and right are not of type Number
        let error = "Add operation can only add numbers";
        expression_arms_of_type(meta, &self.left, &self.right, Type::Num, tok, error);
        Ok(())
    }
}