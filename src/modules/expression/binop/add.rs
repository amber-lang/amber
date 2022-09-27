use heraclitus_compiler::prelude::*;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Add {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Add {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Add {
    syntax_name!("Add");

    fn new() -> Self {
        Add {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "+")?;
        let tok = meta.get_current_token();
        token(meta, "+")?;
        syntax(meta, &mut *self.right)?;
        // If left and right are not of type Number
        let error = "Add operation can only add numbers or text";
        let l_type = self.left.get_type();
        let r_type = self.right.get_type();
        self.kind = expression_arms_of_type(meta, &l_type, &r_type, &[Type::Num, Type::Text], tok, error)?;
        Ok(())
    }
}

impl TranslateModule for Add {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        if self.kind == Type::Text {
            format!("{}{}", left, right)
        }
        else {
            translate_computation(meta, ArithOp::Add, Some(left), Some(right))
        }
    }
}