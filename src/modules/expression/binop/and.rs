use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;
use super::{super::expr::Expr, Binop};

#[derive(Debug)]
pub struct And {
    left: Box<Expr>,
    right: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for And {
    fn new() -> Self {
        And {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Binop::parse_left_expr(meta, &mut *self.left, "and")?;
        token(meta, "and")?;
        syntax(meta, &mut *self.right)?;
        Ok(())
    }
}