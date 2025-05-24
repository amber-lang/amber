use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::compare::{translate_lexical_comparison, translate_array_lexical_comparison, ComparisonOperator};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::modules::types::{Typed, Type};
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Lt {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Lt {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Lt {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "<")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Lt {
    syntax_name!("Lt");

    fn new() -> Self {
        Lt {
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

impl TranslateModule for Lt {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.left.get_type() == Type::array_of(Type::Num) {
            translate_array_lexical_comparison(meta, ComparisonOperator::Lt, &self.left, &self.right)
        } else if [Type::Text, Type::array_of(Type::Text)].contains(&self.left.get_type()) {
            translate_lexical_comparison(meta, ComparisonOperator::Lt, &self.left, &self.right)
        } else {
            let left = self.left.translate(meta);
            let right = self.right.translate(meta);
            translate_computation(meta, ArithOp::Lt, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Lt {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
