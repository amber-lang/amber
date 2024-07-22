use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;
use crate::modules::statement::stmt::Statement;

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
            let mut block = Block::new();
            // Handle comments and empty lines
            if token_by(meta, |token| token.starts_with("//") || token.starts_with('\n')).is_ok() {
                continue
            }
            // Handle else keyword
            if token(meta, "else").is_ok() {
                match token(meta, "{") {
                    Ok(_) => {
                        let mut false_block = Box::new(Block::new());
                        syntax(meta, &mut *false_block)?;
                        self.false_block = Some(false_block);
                        token(meta, "}")?;
                    }
                    Err(_) => {
                        let mut statement = Statement::new();
                        token(meta, ":")?;
                        syntax(meta, &mut statement)?;
                        self.false_block = Some(Box::new(Block::new()));
                        self.false_block.as_mut().unwrap().push_statement(statement);
                    }
                }
                token(meta, "}")?;
                return Ok(())
            }
            if token(meta, "}").is_ok() {
                return Ok(())
            }
            // Handle end of the if chain
            syntax(meta, &mut cond)?;
            match token(meta, "{") {
                Ok(_) => {
                    syntax(meta, &mut block)?;
                    token(meta, "}")?;
                    self.cond_blocks.push((cond, block));
                }
                Err(_) => {
                    let mut statement = Statement::new();
                    token(meta, ":")?;
                    syntax(meta, &mut statement)?;
                    block.push_statement(statement);
                    self.cond_blocks.push((cond, block));
                }
            }
        }
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
            result.push("else".to_string());
            result.push(false_block.translate(meta));
        }
        result.push("fi".to_string());
        result.join("\n")
    }
}

impl DocumentationModule for IfChain {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
