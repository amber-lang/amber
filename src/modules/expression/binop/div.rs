use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::compute::ArithOp;
use crate::translate::compute::translate_float_computation;
use crate::modules::types::{Typed, Type};

use super::BinOp;

#[derive(Debug, Clone)]
pub struct Div {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Div {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl BinOp for Div {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "/")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Div {
    syntax_name!("Div");

    fn new() -> Self {
        Div {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.kind = Self::typecheck_allowed_types(meta, "division", &self.left, &self.right, &[
            Type::Num,
            Type::Int
        ])?;
        Ok(())
    }
}

impl TranslateModule for Div {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        match self.kind {
            Type::Int => ArithmeticFragment::new(left, ArithOp::Div, right).to_frag(),
            _ => translate_float_computation(meta, ArithOp::Div, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Div {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
