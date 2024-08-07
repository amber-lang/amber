use heraclitus_compiler::prelude::*;
use crate::utils::{metadata::ParserMetadata, TranslateMetadata};
use crate::translate::{compute::{translate_computation, ArithOp}, module::TranslateModule};
use crate::modules::types::{Type, Typed};
use crate::docs::module::DocumentationModule;
use super::super::expr::Expr;
use crate::error_type_match;
use super::UnOp;

#[derive(Debug, Clone)]
pub struct Not {
    expr: Box<Expr>
}

impl Typed for Not {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl UnOp for Not {
    fn set_expr(&mut self, expr: Expr) {
        self.expr = Box::new(expr);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "not")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Not {
    syntax_name!("Not");

    fn new() -> Self {
        Not {
            expr: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if !matches!(self.expr.get_type(), Type::Bool) {
            let msg = self.expr.get_error_message(meta);
            return error_type_match!(meta, msg, "logically negate", (self.expr), [Bool])
        }
        Ok(())
    }
}

impl TranslateModule for Not {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let expr = self.expr.translate(meta);
        translate_computation(meta, ArithOp::Not, None, Some(expr))
    }
}

impl DocumentationModule for Not {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
