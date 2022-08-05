use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, Binop};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Lt {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Lt {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Lt {
    syntax_name!("Lt");

    fn new() -> Self {
        Lt {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Binop::parse_left_expr(meta, &mut *self.left, "<")?;
        token(meta, "<")?;
        syntax(meta, &mut *self.right)?;
        Ok(())
    }
}