use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, error::get_error_logger, TranslateMetadata}};
use crate::translate::module::TranslateModule;
use super::statement::statement::Statement;

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>
}

impl Block {
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
        meta.var_mem.push_scope();
        loop {
            match meta.get_current_token() {
                Some(token) => {
                    if ["\n", ";"].contains(&token.word.as_str()) {
                        meta.increment_index();
                        continue;
                    }
<<<<<<< Updated upstream
=======
                    // Handle comments
                    if token.word.starts_with("#") {
                        meta.increment_index();
                        continue
                    }
                    // Handle block end
                    else if token.word == "}" {
                        break;
                    }
>>>>>>> Stashed changes
                }
                None => break
            }
            let mut statemant = Statement::new();
            if let Err(details) = statemant.parse(meta) {
                self.error(meta, details);
            }
            self.statements.push(statemant);
        }
        meta.var_mem.pop_scope();
        Ok(())
    }
}

impl TranslateModule for Block {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        self.statements.iter()
            .map(|module| module.translate(meta))
            .collect::<Vec<_>>().join(";\n")
    }
}