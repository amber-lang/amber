use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::block::Block;

#[derive(Debug, Clone)]
pub struct IfChain {
    cond_blocks: Vec<(Expr, Block)>,
    false_block: Option<Box<Block>>
}

impl SyntaxModule<ParserMetadata> for IfChain {
    syntax_name!("If Condition");

    fn new() -> Self {
        IfChain {
            cond_blocks: vec![],
            false_block: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "if")?;
        // Parse true block
        token(meta, "{")?;
        loop {
            let mut cond = Expr::new();
            let mut block = Block::new().with_needs_noop().with_condition();
            // Handle comments and empty lines
            if token_by(meta, |token| token.starts_with("//") || token.starts_with('\n')).is_ok() {
                continue
            }
            // Handle else keyword
            if token(meta, "else").is_ok() {
                let mut false_block = Box::new(Block::new().with_needs_noop().with_condition());
                syntax(meta, &mut *false_block)?;
                self.false_block = Some(false_block);
                if token(meta, "}").is_err() {
                  return error!(meta, meta.get_current_token(), "Expected `else` condition to be the last in the if chain")?
                }
                return Ok(())
            }
            // Handle end of the if chain
            if token(meta, "}").is_ok() {
                return Ok(())
            }
            syntax(meta, &mut cond)?;
            syntax(meta, &mut block)?;

            self.cond_blocks.push((cond, block));
        }
    }
}

impl TypeCheckModule for IfChain {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Type-check all condition-block pairs
        for (cond, block) in &mut self.cond_blocks {
            cond.typecheck(meta)?;
            block.typecheck(meta)?;
        }

        // Type-check the false block if it exists
        if let Some(false_block) = &mut self.false_block {
            false_block.typecheck(meta)?;
        }

        Ok(())
    }
}

impl TranslateModule for IfChain {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let mut result = vec![];
        let mut is_first = true;
        for (cond, block) in self.cond_blocks.iter() {
            if is_first {
                result.push(fragments!("if [ ", cond.translate(meta), " != 0 ]; then"));
                result.push(block.translate(meta));
                is_first = false;
            } else {
                result.push(fragments!("elif [ ", cond.translate(meta), " != 0 ]; then"));
                result.push(block.translate(meta));
            }
        }
        if let Some(false_block) = &self.false_block {
            result.push(fragments!("else"));
            result.push(false_block.translate(meta));
        }
        result.push(fragments!("fi"));
        BlockFragment::new(result, false).to_frag()
    }
}

impl DocumentationModule for IfChain {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
