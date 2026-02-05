use crate::docs::module::DocumentationModule;
use crate::fragments;
use crate::modules::prelude::*;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Break {
    tok: Option<Token>,
}

impl SyntaxModule<ParserMetadata> for Break {
    syntax_name!("Break");

    fn new() -> Self {
        Break { tok: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.tok = meta.get_current_token();
        token(meta, "break")?;
        Ok(())
    }
}

impl TypeCheckModule for Break {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Detect if the break statement is inside a loop
        if !meta.context.is_loop_ctx {
            return error!(
                meta,
                self.tok.clone(),
                "Break statement can only be used inside a loop"
            );
        }
        Ok(())
    }
}

impl TranslateModule for Break {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("break")
    }
}

impl DocumentationModule for Break {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
