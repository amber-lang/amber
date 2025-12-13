use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
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
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
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

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for And {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        Self::typecheck_allowed_types(meta, "logical AND", &mut self.left, &mut self.right, &[
            Type::Bool,
        ])?;
        Ok(())
    }
}

impl TranslateModule for And {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        ArithmeticFragment::new(left, ArithOp::And, right).to_frag()
    }
}

impl DocumentationModule for And {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
