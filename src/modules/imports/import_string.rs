use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImportString {
    pub value: String,
}

impl ImportString {
    fn resolve_path(&mut self, meta: &ParserMetadata, tok: Option<Token>) -> SyntaxResult {
        if self.value.starts_with("std/") {
            return Ok(());
        }
        let mut path = meta
            .context
            .path
            .as_ref()
            .map_or_else(|| Path::new("."), |path| Path::new(path))
            .to_path_buf();
        path.pop();
        path.push(&self.value);
        match path.to_str() {
            Some(path) => {
                self.value = path.to_string();
                Ok(())
            }
            None => error!(meta, tok => {
                message: format!("Could not resolve path '{}'", path.display()),
                comment: "Path is not valid UTF-8"
            }),
        }
    }
}

impl SyntaxModule<ParserMetadata> for ImportString {
    syntax_name!("Import String");

    fn new() -> Self {
        Self {
            value: String::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        let value = token_by(meta, |word| word.starts_with('"'))?;
        if value.ends_with('"') {
            self.value = value[1..value.len() - 1].to_string();
            self.resolve_path(meta, tok)?;
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
