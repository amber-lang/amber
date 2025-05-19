use heraclitus_compiler::prelude::*;
use crate::fragments;
use crate::modules::prelude::*;
use crate::utils::context::Context;
use crate::modules::block::Block;

#[derive(Debug, Clone)]
pub struct InfiniteLoop {
    block: Block,
}

impl SyntaxModule<ParserMetadata> for InfiniteLoop {
    syntax_name!("Infinite Loop");

    fn new() -> Self {
        InfiniteLoop {
            block: Block::new().with_needs_noop(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "loop")?;
        token(meta, "{")?;
        // Save loop context state and set it to true
        meta.with_context_fn(Context::set_is_loop_ctx, true, |meta| {
            // Parse loop
            syntax(meta, &mut self.block)?;
            token(meta, "}")?;
            Ok(())
        })?;
        Ok(())
    }
}

impl TranslateModule for InfiniteLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        BlockFragment::new(vec![
            fragments!("while :"),
            fragments!("do"),
            self.block.translate(meta),
            fragments!("done")
        ], false).to_frag()
    }
}

impl DocumentationModule for InfiniteLoop {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
