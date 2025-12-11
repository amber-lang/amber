use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::translate::compare::translate_array_equality;
use crate::translate::compute::{translate_float_computation, ArithOp};
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
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
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

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Neq {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        Self::typecheck_equality(meta, &mut self.left, &mut self.right)?;
        Ok(())
    }
}

impl TranslateModule for Neq {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta).with_quotes(false);
        let right = self.right.translate(meta).with_quotes(false);
        match self.left.get_type() {
            Type::Int => ArithmeticFragment::new(left, ArithOp::Neq, right).to_frag(),
            Type::Num => translate_float_computation(meta, ArithOp::Neq, Some(left), Some(right)),
            Type::Array(_) => {
                if let (FragmentKind::VarExpr(left), FragmentKind::VarExpr(right)) = (left, right) {
                    translate_array_equality(left, right, true)
                } else {
                    unreachable!("Arrays are always represented as variable expressions when used as values")
                }
            }
            _ => SubprocessFragment::new(fragments!("[ \"_", left, "\" == \"_", right, "\" ]; echo $?")).to_frag()
        }
    }
}

impl DocumentationModule for Neq {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
