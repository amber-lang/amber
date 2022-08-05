use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, Binop};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Ge {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Ge {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Ge {
    syntax_name!("Ge");

    fn new() -> Self {
        Ge {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Binop::parse_left_expr(meta, &mut *self.left, ">=")?;
        token(meta, ">=")?;
        syntax(meta, &mut *self.right)?;
        Ok(())
    }
}