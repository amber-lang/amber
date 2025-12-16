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
        let old_chain = std::mem::take(&mut self.cond_blocks);
        let mut new_chain = Vec::new();
        let mut chain_deadcode = false;

        for (mut cond, mut block) in old_chain {
            if chain_deadcode {
                continue;
            }

            cond.typecheck(meta)?;
            match cond.analyze_control_flow() {
                Some(true) => {
                    let (facts, _) = cond.extract_facts();
                    meta.with_narrowed_scope(facts, |meta| {
                        block.typecheck(meta)
                    })?;
                    new_chain.push((cond, block));
                    chain_deadcode = true;
                    self.false_block = None;
                },
                Some(false) => {
                    // Dead block, drop it
                },
                None => {
                    let (facts, _) = cond.extract_facts();
                    meta.with_narrowed_scope(facts, |meta| {
                        block.typecheck(meta)
                    })?;
                    new_chain.push((cond, block));
                }
            }
        }

        self.cond_blocks = new_chain;

        if !chain_deadcode {
            if let Some(false_block) = &mut self.false_block {
                false_block.typecheck(meta)?;
            }
        }

        Ok(())
    }
}

impl TranslateModule for IfChain {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.cond_blocks.is_empty() {
            if let Some(false_block) = &self.false_block {
                return false_block.translate(meta);
            }
            return FragmentKind::Empty;
        }
        
        // In case of when only the first condition is true, we can just leave the truth block without any condition
        if let Some((first_cond, first_block)) = self.cond_blocks.first() {
            if first_cond.analyze_control_flow() == Some(true) {
                return first_block.translate(meta);
            }
        }

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
