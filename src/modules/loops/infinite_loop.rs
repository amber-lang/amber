use heraclitus_compiler::prelude::*;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;

#[derive(Debug, Clone)]
pub struct InfiniteLoop {
    block: Block,
}

impl SyntaxModule<ParserMetadata> for InfiniteLoop {
    syntax_name!("Infinite Loop");

    fn new() -> Self {
        InfiniteLoop {
            block: Block::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "loop")?;
        token(meta, "{")?;
        // Save loop context state and set it to true
        let ctx = meta.loop_ctx;
        meta.loop_ctx = true;
        // Parse loop
        syntax(meta, &mut self.block)?;
        token(meta, "}")?;
        // Restore loop context state
        meta.loop_ctx = ctx;
        Ok(())
    }
}

impl TranslateModule for InfiniteLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        vec![
            "while :".to_string(),
            "do".to_string(),
            self.block.translate(meta),
            "done".to_string()
        ].join("\n")
    }
}