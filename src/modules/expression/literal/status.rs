use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::types::{Type, Typed}};
use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone)]
pub struct Status;

impl Typed for Status {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Status {
    syntax_name!("Status");

    fn new() -> Self {
        Status
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "status")?;
        Ok(())        
    }
}

impl TranslateModule for Status {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "$__AS".to_string()
    }
}