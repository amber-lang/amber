use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, error::get_error_logger, TranslateMetadata}};
use crate::translate::module::TranslateModule;
use super::statement::stmt::Statement;

#[derive(Debug, Clone)]
pub struct Block {
    statements: Vec<Statement>
}

impl Block {
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    pub fn push_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    fn error(&mut self, meta: &mut ParserMetadata, details: ErrorDetails) {
        get_error_logger(meta, details)
            .attach_message("Undefined syntax")
            .show()
            .exit()
    }
}

impl SyntaxModule<ParserMetadata> for Block {
    syntax_name!("Block");

    fn new() -> Self {
        Block {
            statements: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        meta.mem.push_scope();
        while let Some(token) = meta.get_current_token() {
            // Handle the end of line or command
            if ["\n", ";"].contains(&token.word.as_str()) {
                meta.increment_index();
                continue;
            }
            // Handle comments
            if token.word.starts_with('#') {
                meta.increment_index();
                continue
            }
            // Handle block end
            else if token.word == "}" {
                break;
            }
            let mut statemant = Statement::new();
            if let Err(details) = statemant.parse(meta) {
                self.error(meta, details);
            }
            self.statements.push(statemant);
        }
        meta.mem.pop_scope();
        Ok(())
    }
}

impl TranslateModule for Block {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        meta.increase_indent();
        let result = if self.is_empty() {
            ":".to_string()
        }
        else {
            self.statements.iter()
                .map(|module| meta.gen_indent() + &module.translate(meta))
                .collect::<Vec<_>>().join(";\n")
        };
        meta.decrease_indent();
        result
    }
}