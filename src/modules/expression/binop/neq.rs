use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::modules::expression::expr::Expr;
use super::BinOp;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Neq {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Neq {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Neq {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "!=")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Neq {
    syntax_name!("Neq");

    fn new() -> Self {
        Neq {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Self::typecheck_equality(meta, &self.left, &self.right)?;
        Ok(())
    }
}

impl TranslateModule for Neq {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta).with_quotes(false);
        let right = self.right.translate(meta).with_quotes(false);
        if self.left.get_type() == Type::Text && self.right.get_type() == Type::Text {
            SubprocessFragment::new(fragments!("[ \"_", left, "\" == \"_", right, "\" ]; echo $?")).to_frag()
        } else {
            translate_computation(meta, ArithOp::Neq, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Neq {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
