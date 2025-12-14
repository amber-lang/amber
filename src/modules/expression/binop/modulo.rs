use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::translate::compute::{ArithOp, translate_float_computation};
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Modulo {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Modulo {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl BinOp for Modulo {
    fn set_left(&mut self, left: Expr) {
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "%")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Modulo {
    syntax_name!("Modulo");

    fn new() -> Self {
        Modulo {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Generic
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Modulo {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        self.kind = Self::typecheck_allowed_types(meta, "modulo", &mut self.left, &mut self.right, &[
            Type::Num,
            Type::Int,
        ])?;
        Ok(())
    }
}

impl TranslateModule for Modulo {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        match self.kind {
            Type::Int => FragmentKind::Arithmetic(ArithmeticFragment::new(left, ArithOp::Modulo, right)),
            Type::Num => translate_float_computation(meta, ArithOp::Modulo, Some(left), Some(right)),
            _ => unreachable!("Unsupported type {} in subtraction operation", self.kind),
        }
    }
}

impl DocumentationModule for Modulo {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
