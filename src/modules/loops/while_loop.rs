use crate::fragments;
use crate::modules::block::Block;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::context::Context;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct WhileLoop {
    condition: Expr,
    block: Block,
}

impl SyntaxModule<ParserMetadata> for WhileLoop {
    syntax_name!("While Loop");

    fn new() -> Self {
        WhileLoop {
            condition: Expr::new(),
            block: Block::new().with_needs_noop().with_condition(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "while")?;
        // Parse the condition expression
        let tok = meta.get_current_token();
        syntax(meta, &mut self.condition)?;

        // Validate that the condition is a boolean expression
        if self.condition.get_type() != Type::Bool {
            return error!(
                meta,
                tok,
                format!(
                    "Expected boolean expression in while condition, got {}",
                    self.condition.get_type()
                )
            );
        }

        syntax(meta, &mut self.block)?;
        Ok(())
    }
}

impl TypeCheckModule for WhileLoop {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.condition.typecheck(meta)?;
        // Save loop context state and set it to true
        meta.with_context_fn(Context::set_is_loop_ctx, true, |meta| {
            self.block.typecheck(meta)
        })?;
        Ok(())
    }
}

impl TranslateModule for WhileLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let result = vec![
            fragments!("while [ ", self.condition.translate(meta), " != 0 ]; do"),
            self.block.translate(meta),
            fragments!("done"),
        ];
        BlockFragment::new(result, false).to_frag()
    }
}

impl DocumentationModule for WhileLoop {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
