use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;

#[derive(Debug)]
pub struct IfCondition {
    expr: Box<Expr>,
    true_block: Box<Block>,
    false_block: Option<Box<Block>>
}

impl SyntaxModule<ParserMetadata> for IfCondition {
    syntax_name!("If Condition");

    fn new() -> Self {
        IfCondition {
            expr: Box::new(Expr::new()),
            true_block: Box::new(Block::new()),
            false_block: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "if")?;
        // Parse expression
        syntax(meta, &mut *self.expr)?;
        // Parse true block
        token(meta, "{")?;
        syntax(meta, &mut *self.true_block)?;
        token(meta, "}")?;
        // Parse false block
        if token(meta, "else").is_ok() {
            token(meta, "{")?;
            let mut false_block = Box::new(Block::new());
            syntax(meta, &mut *false_block)?;
            self.false_block = Some(false_block);
            token(meta, "}")?;
        }
        Ok(())
    }
}

impl TranslateModule for IfCondition {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        result.push(format!("if [ {} != 0 ]; then", self.expr.translate(meta)));
        result.push(self.true_block.translate(meta));
        if let Some(false_block) = &self.false_block {
            result.push("else".to_string());
            result.push(false_block.translate(meta));
        }
        result.push("fi".to_string());
        result.join("\n")
    }
}