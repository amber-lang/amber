use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;

#[derive(Debug)]
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
        let mut is_else = false;
        token(meta, "if")?;
        // Parse true block
        token(meta, "{")?;
        loop {
            let mut cond = Expr::new();
            let mut block = Block::new();
            cond.cannot_fail();
            // Handle else keyword
            if let Ok(_) = token(meta, "else") {
                is_else = true;
                break;
            }
            // Handle end of the if chain
            if let Err(_) = syntax(meta, &mut cond) {
                println!("{:?}", meta.get_current_token());
                break
            }
            token(meta, "{")?;
            syntax(meta, &mut block)?;
            token(meta, "}")?;
            self.cond_blocks.push((cond, block));
        }
        // Parse false block
        if is_else {
            token(meta, "{")?;
            let mut false_block = Box::new(Block::new());
            syntax(meta, &mut *false_block)?;
            self.false_block = Some(false_block);
            token(meta, "}")?;
        }
        token(meta, "}")?;
        Ok(())
    }
}

impl TranslateModule for IfChain {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        let mut is_first = true;
        for (cond, block) in self.cond_blocks.iter() {
            if is_first {
                result.push(format!("if [ {} != 0 ]; then", cond.translate(meta)));
                result.push(block.translate(meta));
                is_first = false;
            } else {
                result.push(format!("elif [ {} != 0 ]; then", cond.translate(meta)));
                result.push(block.translate(meta));
            }
        }
        if let Some(false_block) = &self.false_block {
            result.push(format!("else"));
            result.push(false_block.translate(meta));
        }
        result.push(format!("fi"));
        result.join("\n")
    }
}