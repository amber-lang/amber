use heraclitus_compiler::prelude::*;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
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
        let tok = meta.get_current_token();
        token(meta, "*")?;
        syntax(meta, &mut *self.right)?;
        let error = "Multiply operation can only multiply numbers";
        expression_arms_of_type(meta, &self.left, &self.right, Type::Num, tok, error);
        Ok(())
    }
}

impl TranslateModule for Mul {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Mul, Some(left), Some(right))
    }
}