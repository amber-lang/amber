use heraclitus_compiler::prelude::*;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::import_string::ImportString;

#[derive(Debug, Clone)]
pub struct ImportFile {
    path: ImportString
}

impl SyntaxModule<ParserMetadata> for ImportFile {
    syntax_name!("Import File");

    fn new() -> Self {
        Self {
            path: ImportString::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "import")?;
        syntax(meta, &mut self.path)?;
        Ok(())
    }
}

impl TranslateModule for ImportFile {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        format!("echo {}", self.path.value)
    }
}