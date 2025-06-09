use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::modules::types::{Typed, Type};

use super::BinOp;


#[derive(Debug, Clone)]
pub struct And {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for And {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for And {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "and")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for And {
    syntax_name!("And");

    fn new() -> Self {
        And {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Self::typecheck_equality(meta, &self.left, &self.right)?;
        Ok(())
    }
}

impl TranslateModule for And {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        translate_computation(meta, ArithOp::And, Some(left), Some(right))
    }
}

impl DocumentationModule for And {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
