use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type};
use crate::modules::{Type, Typed};


#[derive(Debug)]
pub struct And {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for And {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for And {
    syntax_name!("And");

    fn new() -> Self {
        And {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.left, "and")?;
        let tok = meta.get_current_token();
        token(meta, "and")?;
        syntax(meta, &mut *self.right)?;
        let error = "Logical and operation can only be used on arguments of the same type";
        expression_arms_of_same_type(meta, &self.left, &self.right, tok, error);
        Ok(())
    }
}