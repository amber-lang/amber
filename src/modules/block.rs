use std::process::exit;

use heraclitus_compiler::prelude::*;
use super::statement::Statement;

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>
}

impl Block {
    fn error(&mut self, meta: &mut DefaultMetadata, mut details: ErrorDetails) {
        if let Some(path) = meta.path.clone() {
            if let Ok(location) = details.get_pos_by_file(&path) {
                Logger::new_err(path, location)
                    .attach_message("Undefined syntax")
                    .show()
                    .exit();
            } else {
                println!("Couldn't load file '{}'", path);
                exit(1);
            }   
        }
    }
}

impl SyntaxModule<DefaultMetadata> for Block {
    fn new() -> Self {
        Block {
            statements: vec![]
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        loop {
            if let None = meta.get_token_at(meta.get_index()) {
                break;
            }
            let mut statemant = Statement::new();
            if let Err(details) = statemant.parse(meta) {
                self.error(meta, details);
            }
            self.statements.push(statemant);
        }
        Ok(())
    }
}