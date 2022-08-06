use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, parse_left_expr};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Sub {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Sub {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Sub {
    syntax_name!("Sub");

    fn new() -> Self {
        Sub {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Void
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.left, "-")?;
        token(meta, "-")?;
        syntax(meta, &mut *self.right)?;
        Ok(())
    }
}