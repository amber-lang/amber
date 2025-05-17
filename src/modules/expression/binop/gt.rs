use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::compare::{translate_comparison, ComparisonOperator};
use crate::translate::compute::{ArithOp, translate_computation};
use crate::modules::types::{Typed, Type};
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Gt {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Gt {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Gt {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, ">")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Gt {
    syntax_name!("Gt");

    fn new() -> Self {
        Gt {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Self::typecheck_allowed_types(meta, "compare", &self.left, &self.right, &[Type::Num])?;
        Ok(())
    }
}

impl TranslateModule for Gt {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.left.get_type() == Type::array_of(Type::Num) {
            translate_comparison(meta, ComparisonOperator::Gt, &self.left, &self.right)
        } else {
            let left = self.left.translate(meta);
            let right = self.right.translate(meta);
            translate_computation(meta, ArithOp::Gt, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Gt {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
