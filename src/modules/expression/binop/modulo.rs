use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::compute::{ArithOp, translate_computation};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Modulo {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Modulo {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Modulo {
    syntax_name!("Modulo");

    fn new() -> Self {
        Modulo {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "%")?;
        let tok = meta.get_current_token();
        token(meta, "%")?;
        syntax(meta, &mut *self.right)?;
        let error = "Modulo operation can only be applied to numbers";
        let l_type = self.left.get_type();
        let r_type = self.right.get_type();
        let predicate = |kind| matches!(kind, Type::Num);
        expression_arms_of_type(meta, &l_type, &r_type, predicate, tok, error)?;
        Ok(())
    }
}

impl TranslateModule for Modulo {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::Modulo, Some(left), Some(right))
    }
}

impl DocumentationModule for Modulo {
    fn document(&self) -> String {
        "".to_string()
    }
}
