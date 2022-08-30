use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::error::get_error_logger;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;
use crate::modules::statement::st::{Statement, StatementType};

#[derive(Debug)]
pub struct IfCondition {
    expr: Box<Expr>,
    true_block: Box<Block>,
    false_block: Option<Box<Block>>,
}

impl IfCondition {
    fn translate_block(&self, meta: &mut TranslateMetadata, block: &Block) -> String {
        if block.is_empty() {
            ":\n".to_string()
        } else {
            block.translate(meta)
        }
    }

    fn prevent_not_using_if_chain(&self, meta: &mut ParserMetadata, statement: &Statement, tok: Option<Token>) {
        let is_not_using_if_chain = match statement.value.as_ref().unwrap() {
            StatementType::IfCondition(_) => true,
            StatementType::IfChain(_) => true,
            _ => false
        };
        if is_not_using_if_chain {
            let details = ErrorDetails::from_token_option(tok);
            // TODO: [A34] Add a comment pointing to the website documentation
            get_error_logger(meta, details)
                .attach_message("You should use if-chain instead of nested if else statements")
                .show()
                .exit()
        }
    }
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
        match token(meta, "{") {
            Ok(_) => {
                syntax(meta, &mut *self.true_block)?;
                token(meta, "}")?;
            }
            Err(_) => {
                let mut statement = Statement::new();
                token(meta, "=>")?;
                syntax(meta, &mut statement)?;
                self.true_block.push_statement(statement);
            }
        }
        // Parse false block
        if token(meta, "else").is_ok() {
            match token(meta, "{") {
                Ok(_) => {
                    let mut false_block = Box::new(Block::new());
                    syntax(meta, &mut *false_block)?;
                    self.false_block = Some(false_block);
                    token(meta, "}")?;
                }
                Err(_) => {
                    token(meta, "=>")?;
                    let tok = meta.get_current_token();
                    let mut statement = Statement::new();
                    syntax(meta, &mut statement)?;
                    // Check if the statement is using if chain syntax sugar
                    self.prevent_not_using_if_chain(meta, &statement, tok);
                    self.false_block.get_or_insert(Box::new(Block::new())).push_statement(statement);
                }
            }
        }
        Ok(())
    }
}

impl TranslateModule for IfCondition {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        result.push(format!("if [ {} != 0 ]; then", self.expr.translate(meta)));
        result.push(self.translate_block(meta, &self.true_block));
        if let Some(false_block) = &self.false_block {
            result.push("else".to_string());
            result.push(self.translate_block(meta, false_block));
        }
        result.push("fi".to_string());
        result.join("\n")
    }
}