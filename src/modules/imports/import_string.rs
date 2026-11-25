use crate::{compiler::SourcePath, utils::ParserMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct ImportString {
    pub value: String,
    pub source_path: SourcePath,
}

impl ImportString {

    pub fn resolve_path(&mut self, meta: &ParserMetadata, tok: Option<Token>) -> SyntaxResult {
        if self.value.starts_with("std/") {
            self.source_path = SourcePath::StdLib(self.value.clone());
            return Ok(());
        }
        // Standard library modules can only import other standard library modules
        let Some(path) = meta.context.path.join_relative(&self.value) else {
            return error!(meta, tok => {
                message: format!("Could not resolve path '{}'", self.value),
                comment: "Standard library modules cannot import files by relative path"
            });
        };
        let Some(path) = path.to_str() else {
            return error!(meta, tok => {
                message: format!("Could not resolve path '{}'", path.display()),
                comment: "Path is not valid UTF-8"
            });
        };
        match SourcePath::file(path) {
            Ok(source_path) => {
                self.source_path = source_path;
                Ok(())
            }
            Err(err) => error!(meta, tok => {
                message: format!("Could not read file '{}'", path),
                comment: err.to_string()
            }),
        }
    }
}

impl SyntaxModule<ParserMetadata> for ImportString {
    syntax_name!("Import String");

    fn new() -> Self {
        Self {
            value: String::new(),
            source_path: SourcePath::Stdin
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |word| word.starts_with('"'))?;
        if value.ends_with('"') {
            self.value = value[1..value.len() - 1].to_string();
        } else {
            return error!(
                meta,
                meta.get_current_token(),
                "Import string cannot interpolate expressions"
            );
        }
        Ok(())
    }
}
