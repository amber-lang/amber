use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
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
        let tok = meta.get_current_token();
        token(meta, "-")?;
        syntax(meta, &mut *self.right)?;
        let error = "Substract operation can only substract numbers";
        expression_arms_of_type(meta, &self.left, &self.right, Type::Num, tok, error);
        Ok(())
    }
}