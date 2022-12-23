use heraclitus_compiler::prelude::*;
use crate::modules::block::Block;
use crate::modules::statement::stmt::Statement;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Silent {
    block: Box<Block>
}

impl SyntaxModule<ParserMetadata> for Silent {
    syntax_name!("Silent");

    fn new() -> Self {
        Silent {
            block: Box::new(Block::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "silent")?;
        match token(meta, "{") {
            Ok(_) => {
                syntax(meta, &mut *self.block)?;
                token(meta, "}")?;
            },
            Err(_) => {
                let mut statement = Statement::new();
                syntax(meta, &mut statement)?;
                self.block.push_statement(statement);
            }
        }
        Ok(())
    }
}

impl TranslateModule for Silent {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        meta.silenced = true;
        let translated = self.block.translate(meta);
        meta.silenced = false;
        translated
    }
}