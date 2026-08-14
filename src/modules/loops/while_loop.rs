use crate::fragments;
use crate::modules::block::Block;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::context::Context;
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "while"]
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
        syntax(meta, &mut self.condition)?;
        syntax(meta, &mut self.block)?;
        Ok(())
    }
}

impl TypeCheckModule for WhileLoop {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.condition.typecheck(meta)?;

        if ! matches!(self.condition.get_type(), Type::Bool | Type::Text | Type::Array(_)) {
            return error_pos!(
                meta,
                self.condition.get_position(),
                format!(
                    "Expected boolean expression in while condition, got {}",
                    self.condition.get_type()
                )
            );
        }

        meta.with_context_fn(Context::set_is_loop_ctx, true, |meta| {
            self.block.typecheck(meta)
        })?;
        Ok(())
    }
}

impl TranslateModule for WhileLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let result = vec![
            fragments!("while ", self.condition.translate(meta).with_condition(true), "; do"),
            self.block.translate(meta),
            fragments!("done"),
        ];
        BlockFragment::new(result, false).to_frag()
    }
}

crate::impl_documentation_noop!(WhileLoop);
