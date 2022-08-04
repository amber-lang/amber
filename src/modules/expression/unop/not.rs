use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;
use super::super::expr::Expr;

#[derive(Debug)]
pub struct Not {
    expr: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for Not {
    syntax_name!("Not");

    fn new() -> Self {
        Not {
            expr: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "not")?;
        syntax(meta, &mut *self.expr)?;
        Ok(())
    }
}

