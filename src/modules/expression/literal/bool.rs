use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::metadata::ParserMetadata};
use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone)]
pub struct Bool {
    value: bool
}

impl Bool {
    pub fn get_integer_value(&self) -> Option<isize> {
        let value = if self.value { 1 } else { 0 };
        Some(value)
    }
}

impl Typed for Bool {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Bool {
    syntax_name!("Bool");

    fn new() -> Self {
        Bool {
            value: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |value| ["true", "false"].contains(&value.as_str()))?;
        self.value = value == "true";
        Ok(())
    }
}

impl TranslateModule for Bool {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        format!("{}", if self.value { 1 } else { 0 })
    }
}

impl DocumentationModule for Bool {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
