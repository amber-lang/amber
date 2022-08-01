use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;
use super::expr::Expr;

#[derive(Debug)]
pub struct Parenthesis {
    value: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for Parenthesis {
    fn new() -> Self {
        Parenthesis {
            value: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "(")?;
        syntax(meta, &mut *self.value)?;
        token(meta, ")")?;
        Ok(())
    }
}