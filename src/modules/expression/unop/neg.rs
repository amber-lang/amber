use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{utils::{metadata::ParserMetadata, TranslateMetadata}, modules::types::{Type, Typed}, translate::{module::TranslateModule, compute::{translate_computation, ArithOp}}};
use super::{super::expr::Expr, UnOp};
use crate::docs::module::DocumentationModule;
use crate::error_type_match;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neg {
    expr: Box<Expr>
}

impl Typed for Neg {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl UnOp for Neg {
    fn set_expr(&mut self, expr: Expr) {
        self.expr = Box::new(expr);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "-")?;
        Ok(())
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
        if !matches!(self.expr.get_type(), Type::Num) {
            let msg = self.expr.get_error_message(meta);
            return error_type_match!(meta, msg, "arithmetically negate", (self.expr), [Num])
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
