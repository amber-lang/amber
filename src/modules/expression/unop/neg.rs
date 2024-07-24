use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, TranslateMetadata}, modules::types::{Type, Typed}, translate::{module::TranslateModule, compute::{translate_computation, ArithOp}}};
use super::super::expr::Expr;
use crate::docs::module::DocumentationModule;

#[derive(Debug, Clone)]
pub struct Neg {
    expr: Box<Expr>
}

impl Typed for Neg {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Neg {
    syntax_name!("Neg");

    fn new() -> Self {
        Neg {
            expr: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "-")?;
        let tok = meta.get_current_token();
        syntax(meta, &mut *self.expr)?;
        if ! matches!(self.expr.get_type(), Type::Num) {
            return error!(meta, tok, "Only numbers can be negated");
        }
        Ok(())
    }
}

impl TranslateModule for Neg {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let expr = self.expr.translate(meta);
        translate_computation(meta, ArithOp::Neg, None, Some(expr))
    }
}

impl DocumentationModule for Neg {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
