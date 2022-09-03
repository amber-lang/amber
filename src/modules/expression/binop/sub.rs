use heraclitus_compiler::prelude::*;
use crate::translate::compute::{ArithOp, translate_computation};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::{Type, Typed};

#[derive(Debug)]
pub struct Sub {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Sub {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Sub {
    syntax_name!("Sub");

    fn new() -> Self {
        Sub {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "-")?;
        let tok = meta.get_current_token();
        token(meta, "-")?;
        syntax(meta, &mut *self.right)?;
        let error = "Substract operation can only substract numbers";
        let l_type = self.left.get_type();
        let r_type = self.right.get_type();
        expression_arms_of_type(meta, &l_type, &r_type, &[Type::Num], tok, error);
        Ok(())
    }
}

impl TranslateModule for Sub {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Sub, Some(left), Some(right))
    }
}