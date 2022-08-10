use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};
use crate::translate::module::TranslateModule;
use super::expr::Expr;

#[derive(Debug)]
pub struct Parenthesis {
    value: Box<Expr>,
    kind: Type
}

impl Typed for Parenthesis {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Parenthesis {
    syntax_name!("Parenthesis");

    fn new() -> Self {
        Parenthesis {
            value: Box::new(Expr::new()),
            kind: Type::Void
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "(")?;
        syntax(meta, &mut *self.value)?;
        self.kind = self.value.get_type();
        token(meta, ")")?;
        Ok(())
    }
}

impl TranslateModule for Parenthesis {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        format!("({})", self.value.translate(meta))
    }
}