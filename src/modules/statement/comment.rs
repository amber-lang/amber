use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::docs::module::DocumentationModule;
use crate::utils::metadata::ParserMetadata;
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub value: String
}

impl SyntaxModule<ParserMetadata> for Comment {
    syntax_name!("Comment");

    fn new() -> Self {
        Comment {
            value: String::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |word| word.starts_with("//"))?;
        self.value = value.get(2..).unwrap_or("").trim().to_string();
        Ok(())
    }
}

impl TranslateModule for Comment {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        if meta.minify {
            String::new()
        } else {
            format!("# {}", self.value)
        }
    }
}

impl DocumentationModule for Comment {
    fn document(&self, _meta: &ParserMetadata) -> String {
        self.value.clone() + "\n\n"
    }
}
