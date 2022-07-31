use heraclitus_compiler::prelude::*;
use super::super::expr::{Expr, ExprId};

#[derive(Debug)]
pub struct Add {
    left: Box<Expr>,
    right: Box<Expr>
}

impl SyntaxModule<DefaultMetadata> for Add {
    fn new() -> Self {
        Add {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        self.left.exclude(ExprId::Add);
        syntax(meta, &mut *self.left)?;
        token(meta, "+")?;
        syntax(meta, &mut *self.right)?;
        Ok(())
    }
}