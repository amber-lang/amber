use crate::fragments;
use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Continue;

impl SyntaxModule<ParserMetadata> for Continue {
    syntax_name!("Continue");

    fn new() -> Self {
        Continue
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        token(meta, "continue")?;
        // Detect if the continue statement is inside a loop
        if !meta.context.is_loop_ctx {
            return error!(meta, tok, "Continue statement can only be used inside a loop")
        }
        Ok(())
    }
}

impl TranslateModule for Continue {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("continue")
    }
}

impl DocumentationModule for Continue {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
