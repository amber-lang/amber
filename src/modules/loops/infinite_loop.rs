use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
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
        let mut new_is_loop_ctx = true;
        swap(&mut new_is_loop_ctx, &mut meta.context.is_loop_ctx);
        // Parse loop
        syntax(meta, &mut self.block)?;
        token(meta, "}")?;
        // Restore loop context state
        swap(&mut new_is_loop_ctx, &mut meta.context.is_loop_ctx);
        Ok(())
    }
}

impl TranslateModule for InfiniteLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        ["while :".to_string(),
            "do".to_string(),
            self.block.translate(meta),
            "done".to_string()].join("\n")
    }
}

impl DocumentationModule for InfiniteLoop {
    fn document(&self) -> String {
        "".to_string()
    }
}
