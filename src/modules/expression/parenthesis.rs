use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::metadata::ParserMetadata};
use crate::translate::module::TranslateModule;
use super::expr::Expr;

#[derive(Debug, Clone)]
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

impl TranslateModule for Parenthesis {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        self.value.translate(meta)
    }
}

impl DocumentationModule for Parenthesis {
    fn document(&self) -> String {
        "".to_string()
    }
}
