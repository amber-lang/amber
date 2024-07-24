use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::metadata::ParserMetadata};
use crate::translate::module::TranslateModule;
use super::expr::Expr;

#[derive(Debug, Clone)]
pub struct Parentheses {
    value: Box<Expr>,
    kind: Type
}

impl Typed for Parentheses {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Parentheses {
    syntax_name!("Parentheses");

    fn new() -> Self {
        Parentheses {
            value: Box::new(Expr::new()),
            kind: Type::Null
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

impl TranslateModule for Parentheses {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        self.value.translate(meta)
    }
}

impl DocumentationModule for Parentheses {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
