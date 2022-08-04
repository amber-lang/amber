use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;
use super::{super::expr::Expr, Binop};

#[derive(Debug)]
pub struct Lt {
    left: Box<Expr>,
    right: Box<Expr>
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