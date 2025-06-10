use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::compare::{translate_lexical_comparison, translate_array_lexical_comparison, ComparisonOperator};
use crate::modules::types::{Typed, Type};
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Ge {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Ge {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Ge {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, ">=")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Ge {
    syntax_name!("Ge");

    fn new() -> Self {
        Ge {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Self::typecheck_allowed_types(meta, "comparison", &self.left, &self.right, &[
            Type::Num,
            Type::Text,
            Type::array_of(Type::Num),
            Type::array_of(Type::Text),
        ])?;
        Ok(())
    }
}

impl TranslateModule for Ge {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.left.get_type() == Type::array_of(Type::Num) {
            translate_array_lexical_comparison(meta, ComparisonOperator::Ge, &self.left, &self.right)
        } else if [Type::Text, Type::array_of(Type::Text)].contains(&self.left.get_type()) {
            translate_lexical_comparison(meta, ComparisonOperator::Ge, &self.left, &self.right)
        } else {
            let left = self.left.translate(meta);
            let right = self.right.translate(meta);
            translate_computation(meta, ArithOp::Ge, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Ge {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
