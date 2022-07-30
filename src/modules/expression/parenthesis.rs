use heraclitus_compiler::prelude::*;

use super::expr::Expr;

#[derive(Debug)]
pub struct Parenthesis {
    value: Box<Expr>
}

impl SyntaxModule<DefaultMetadata> for Parenthesis {
    fn new() -> Self {
        Parenthesis {
            value: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        token(meta, "(")?;
        syntax(meta, &mut *self.value)?;
        token(meta, ")")?;
        Ok(())
    }
}