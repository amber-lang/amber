use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::translate::compute::{ArithOp, translate_float_computation};
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};

use super::BinOp;

#[derive(Debug, Clone)]
pub struct Sub {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Sub {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl BinOp for Sub {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "-")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Sub {
    syntax_name!("Sub");

    fn new() -> Self {
        Sub {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.kind = Self::typecheck_allowed_types(meta, "subtraction", &self.left, &self.right, &[
            Type::Num,
            Type::Int,
        ])?;
        Ok(())
    }
}

impl TranslateModule for Sub {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        match self.kind {
            Type::Int => FragmentKind::Arithmetic(ArithmeticFragment::new(left, ArithOp::Sub, right)),
            Type::Num => translate_float_computation(meta, ArithOp::Sub, Some(left), Some(right)),
            _ => unreachable!("Unsupported type {} in subtraction operation", self.kind),
        }
    }
}

impl DocumentationModule for Sub {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
