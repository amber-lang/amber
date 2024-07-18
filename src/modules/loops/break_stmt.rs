use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Break;

impl SyntaxModule<ParserMetadata> for Break {
    syntax_name!("Break");

    fn new() -> Self {
        Break
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        token(meta, "break")?;
        // Detect if the break statement is inside a loop
        if !meta.context.is_loop_ctx {
            return error!(meta, tok, "Break statement can only be used inside a loop")
        }
        Ok(())
    }
}

impl TranslateModule for Break {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "break".to_string()
    }
}

impl DocumentationModule for Break {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
