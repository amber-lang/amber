use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, parse_left_expr};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Mul {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Mul {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Mul {
    syntax_name!("Mul");

    fn new() -> Self {
        Mul {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Void
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.left, "*")?;
        token(meta, "*")?;
        syntax(meta, &mut *self.right)?;
        Ok(())
    }
}