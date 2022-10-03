use heraclitus_compiler::prelude::*;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::types::Type;
use crate::modules::block::Block;

use super::variable::variable_name_extensions;

#[derive(Debug, Clone)]
pub struct Main {
    pub args: Vec<String>,
    pub block: Block,
    pub is_skipped: bool
}

impl SyntaxModule<ParserMetadata> for Main {
    syntax_name!("Main");

    fn new() -> Self {
        Self {
            args: vec![],
            block: Block::new(),
            is_skipped: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        token(meta, "main")?;
        // Main cannot be parsed inside of a block
        if meta.mem.get_depth() > 1 {
            return error!(meta, tok, "Main must be in the global scope")
        }
        // If this main is included in other file, skip it
        if !meta.trace.is_empty() {
            self.is_skipped = true;
        }
        context!({
            if token(meta, "(").is_ok() {
                loop {
                    if token(meta, ")").is_ok() {
                        break;
                    }
                    self.args.push(variable(meta, variable_name_extensions())?);
                    match token(meta, ")") {
                        Ok(_) => break,
                        Err(_) => token(meta, ",")?
                    };
                }
            }
            token(meta, "{")?;
            // Create a new scope for variables
            meta.mem.push_scope();
            // Create variables
            for arg in self.args.iter() {
                meta.mem.add_variable(arg, Type::Text, false);
            }
            // Parse the block
            syntax(meta, &mut self.block)?;
            // Remove the scope made for variables
            meta.mem.pop_scope();
            token(meta, "}")?;
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, "Undefined syntax in main block")
        })
    }
}

impl TranslateModule for Main {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let variables = self.args.iter().enumerate()
            .map(|(index, name)| format!("{name}=${}", index + 1))
            .collect::<Vec<_>>().join("\n");
        if self.is_skipped {
            String::new()
        } else {
            format!("{variables}\n{}", self.block.translate(meta))
        }
    }
}