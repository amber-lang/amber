use heraclitus_compiler::prelude::*;
use crate::utils::{metadata::ParserMetadata, TranslateMetadata};
use crate::translate::{compute::{translate_computation, ArithOp}, module::TranslateModule};
use crate::modules::types::{Type, Typed};
use crate::docs::module::DocumentationModule;
use super::super::expr::Expr;

#[derive(Debug, Clone)]
pub struct Not {
    expr: Box<Expr>
}

impl Typed for Not {
    fn get_type(&self) -> Type {
        Type::Bool
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
        token(meta, "not")?;
        syntax(meta, &mut *self.expr)?;
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
    fn document(&self) -> String {
        "".to_string()
    }
}
