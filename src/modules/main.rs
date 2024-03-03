use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
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
        if !meta.is_global_scope() {
            return error!(meta, tok, "Main must be in the global scope")
        }
        // If this main is included in other file, skip it
        if !meta.context.trace.is_empty() {
            self.is_skipped = true;
        }
        context!({
            meta.context.is_main_ctx = true;
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
            meta.push_scope();
            // Create variables
            for arg in self.args.iter() {
                meta.add_var(arg, Type::Text);
            }
            // Parse the block
            syntax(meta, &mut self.block)?;
            // Remove the scope made for variables
            meta.pop_scope();
            token(meta, "}")?;
            meta.context.is_main_ctx = false;
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

impl DocumentationModule for Main {
    fn document(&self) -> String {
        "".to_string()
    }
}
