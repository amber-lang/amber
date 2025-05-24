use heraclitus_compiler::prelude::*;
use crate::modules::prelude::FragmentKind;
use crate::utils::{metadata::ParserMetadata, TranslateMetadata};
use crate::translate::{compute::{translate_computation, ArithOp}, module::TranslateModule};
use crate::modules::types::{Type, Typed};
use crate::docs::module::DocumentationModule;
use super::super::expr::Expr;
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
        Self::typecheck_allowed_types(meta, "logical negation", &self.expr, &[Type::Bool])?;
        Ok(())
    }
}

impl TranslateModule for Not {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        translate_computation(meta, ArithOp::Not, None, Some(expr))
    }
}

impl DocumentationModule for Not {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
