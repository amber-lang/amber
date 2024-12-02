use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::{ParserMetadata, TranslateMetadata}};
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Null {}

impl Typed for Null {
    fn get_type(&self) -> Type {
        Type::Null
    }
}

impl SyntaxModule<ParserMetadata> for Null {
    syntax_name!("Null");

    fn new() -> Self {
        Null {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "null")?;
        Ok(())
    }
}

impl TranslateModule for Null {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "''".to_string()
    }
}

impl DocumentationModule for Null {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
