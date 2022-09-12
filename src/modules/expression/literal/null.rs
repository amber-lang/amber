use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::types::{Type, Typed}};
use crate::translate::module::TranslateModule;

#[derive(Debug)]
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