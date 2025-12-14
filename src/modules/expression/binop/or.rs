use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::translate::compute::ArithOp;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};

use super::BinOp;

#[derive(Debug, Clone)]
pub struct Or {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Typed for Or {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Or {
    fn set_left(&mut self, left: Expr) {
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "or")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Or {
    syntax_name!("Or");

    fn new() -> Self {
        Or {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Or {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        Self::typecheck_allowed_types(meta, "logical OR", &mut self.left, &mut self.right, &[
            Type::Bool,
        ])?;
        Ok(())
    }
}

impl TranslateModule for Or {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        ArithmeticFragment::new(left, ArithOp::Or, right).to_frag()
    }
}

impl DocumentationModule for Or {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
