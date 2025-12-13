use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};

use super::TypeOp;

#[derive(Debug, Clone)]
pub struct Is {
    expr: Box<Expr>,
    kind: Type
}

impl Typed for Is {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl TypeOp for Is {
    fn set_left(&mut self, left: Expr) {
        *self.expr = left;
    }

    fn set_right(&mut self, right: Type) {
        self.kind = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "is")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Is {
    syntax_name!("Add");

    fn new() -> Self {
        Is {
            expr: Box::new(Expr::new()),
            kind: Type::default()
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Is {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)
    }
}

impl TranslateModule for Is {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        if self.expr.get_type() == self.kind {
            fragments!("1")
        } else {
            fragments!("0")
        }
    }
}

impl DocumentationModule for Is {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
