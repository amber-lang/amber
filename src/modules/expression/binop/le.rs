use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::compare::{translate_lexical_comparison, translate_array_lexical_comparison, ComparisonOperator};
use crate::translate::compute::{ArithOp, translate_float_computation};
use crate::modules::types::{Typed, Type};
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Le {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Le {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Le {
    fn set_left(&mut self, left: Expr) {
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "<=")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Le {
    syntax_name!("Le");

    fn new() -> Self {
        Le {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Le {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        Self::typecheck_allowed_types(meta, "comparison", &mut self.left, &mut self.right, &[
            Type::Num,
            Type::Int,
            Type::Text,
            Type::array_of(Type::Num),
            Type::array_of(Type::Int),
            Type::array_of(Type::Text),
        ])?;
        Ok(())
    }
}

impl TranslateModule for Le {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        match self.left.get_type() {
            Type::Int => {
                let left = self.left.translate(meta).with_quotes(false);
                let right = self.right.translate(meta).with_quotes(false);
                ArithmeticFragment::new(left, ArithOp::Le, right).to_frag()
            }
            Type::Num => {
                let left = self.left.translate(meta);
                let right = self.right.translate(meta);
                translate_float_computation(meta, ArithOp::Le, Some(left), Some(right))
            }
            Type::Array(inner_type) => {
                translate_array_lexical_comparison(meta, ComparisonOperator::Le, &self.left, &self.right, *inner_type)
            }
            Type::Text => {
                translate_lexical_comparison(meta, ComparisonOperator::Le, &self.left, &self.right)
            }
            _ => unreachable!("Unsupported type {} in less equal comparison", self.left.get_type())
        }
    }
}

impl DocumentationModule for Le {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
