use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::unop::UnOp;
use crate::modules::prelude::{RawFragment, FragmentKind, FragmentRenderable};
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
        Self::typecheck_allowed_types(meta, "arithmetic negation", &self.expr, &[Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Neg {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        translate_computation(meta, ArithOp::Neg, None, Some(expr))
    }
}

impl Neg {
    pub fn get_integer_value(&self) -> Option<isize> {
        self.expr.get_integer_value().map(isize::neg)
    }

    pub fn get_array_index(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if let Some(expr) = self.get_integer_value() {
            RawFragment::from(expr.to_string()).to_frag()
        } else {
            self.translate(meta)
        }
    }
}

impl DocumentationModule for Neg {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
