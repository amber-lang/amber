use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};
use super::super::expr::Expr;

#[derive(Debug)]
pub struct Not {
    expr: Box<Expr>,
    kind: Type
}

impl Typed for Not {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Not {
    syntax_name!("Not");

    fn new() -> Self {
        Not {
            expr: Box::new(Expr::new()),
            kind: Type::Bool
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "not")?;
        syntax(meta, &mut *self.expr)?;
        Ok(())
    }
}

