use crate::docs::module::DocumentationModule;
use crate::error_type_match;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::unop::UnOp;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::ParserMetadata;
use crate::utils::TranslateMetadata;
use heraclitus_compiler::prelude::*;
use std::ops::Neg as _;

#[derive(Debug, Clone)]
pub struct Neg {
    expr: Box<Expr>
}

impl Neg {
    pub fn get_integer_value(&self) -> Option<isize> {
        self.expr.get_integer_value().map(isize::neg)
    }

    pub fn get_array_index(&self, meta: &mut TranslateMetadata) -> String {
        if let Some(expr) = self.get_integer_value() {
            expr.to_string()
        } else {
            self.translate(meta)
        }
    }
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
