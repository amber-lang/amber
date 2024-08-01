use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;
use crate::{
    docs::module::DocumentationModule,
    modules::types::{Type, Typed},
    utils::metadata::ParserMetadata,
};
use heraclitus_compiler::prelude::*;

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

impl DocumentationModule for Status {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
