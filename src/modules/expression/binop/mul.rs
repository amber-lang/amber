use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::docs::module::DocumentationModule;
use crate::translate::compute::ArithOp;
use crate::modules::prelude::*;
use crate::translate::compute::translate_float_computation;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};

use super::BinOp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mul {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Mul {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl BinOp for Mul {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "*")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Mul {
    syntax_name!("Mul");

    fn new() -> Self {
        Mul {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.kind = Self::typecheck_allowed_types(meta, "multiplication", &self.left, &self.right, &[
            Type::Num,
            Type::Int,
        ])?;
        Ok(())
    }
}

impl TranslateModule for Mul {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        match self.kind {
            Type::Int => FragmentKind::Arithmetic(ArithmeticFragment::new(left, ArithOp::Mul, right)),
            Type::Num => translate_float_computation(meta, ArithOp::Mul, Some(left), Some(right)),
            _ => unreachable!("Unsupported type {} in multiplication operation", self.kind)
        }
    }
}

impl DocumentationModule for Mul {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
