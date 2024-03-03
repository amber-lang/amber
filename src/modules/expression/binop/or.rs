use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_same_type};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Or {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Or {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Or {
    syntax_name!("Or");

    fn new() -> Self {
        Or {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "or")?;
        let tok = meta.get_current_token();
        token(meta, "or")?;
        syntax(meta, &mut *self.right)?;
        let error = "Logical or operation can only be used on arguments of the same type";
        expression_arms_of_same_type(meta, &self.left, &self.right, tok, error)?;
        Ok(())
    }
}

impl TranslateModule for Or {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Or, Some(left), Some(right))
    }
}

impl DocumentationModule for Or {
    fn document(&self) -> String {
        "".to_string()
    }
}
