use heraclitus_compiler::prelude::*;
use crate::translate::module::TranslateModule;
use crate::utils::error::get_error_logger;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug)]
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
        dbg!(meta.loop_ctx);
        if !meta.loop_ctx {
            let details = ErrorDetails::from_token_option(tok);
            get_error_logger(meta, details)
                .attach_message("Break statement can only be used inside a loop")
                .show()
                .exit();
        }
        Ok(())
    }
}

impl TranslateModule for Break {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "break".to_string()
    }
}
