use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type};
use crate::modules::types::{Typed, Type};


#[derive(Debug, Clone)]
pub struct And {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for And {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for And {
    syntax_name!("And");

    fn new() -> Self {
        And {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "and")?;
        let tok = meta.get_current_token();
        token(meta, "and")?;
        syntax(meta, &mut *self.right)?;
        let error = "Logical and operation can only be used on arguments of the same type";
        expression_arms_of_same_type(meta, &self.left, &self.right, tok, error)?;
        Ok(())
    }
}
impl TranslateModule for And {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::And, Some(left), Some(right))
    }
}

impl DocumentationModule for And {
    fn document(&self) -> String {
        "".to_string()
    }
}
