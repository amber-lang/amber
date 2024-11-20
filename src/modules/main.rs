use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::types::Type;
use crate::modules::block::Block;

use super::variable::variable_name_extensions;

#[derive(Debug, Clone)]
pub struct Main {
    pub args: Option<String>,
    pub block: Block,
    pub is_skipped: bool
}

impl SyntaxModule<ParserMetadata> for Main {
    syntax_name!("Main");

    fn new() -> Self {
        Self {
            args: None,
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
                self.args = Some(variable(meta, variable_name_extensions())?);
                token(meta, ")")?;
            }
            token(meta, "{")?;
            // Create a new scope for variables
            meta.with_push_scope(|meta| {
                // Create variables
                for arg in self.args.iter() {
                    meta.add_var(arg, Type::Array(Box::new(Type::Text)), true);
                }
                // Parse the block
                syntax(meta, &mut self.block)?;
                Ok(())
            })?;
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
        if self.is_skipped {
            String::new()
        } else {
            let quote = meta.gen_quote();
            let dollar = meta.gen_dollar();
            let args = self.args.clone().map_or_else(
                String::new,
                |name| format!("declare -r {name}=({quote}{dollar}0{quote} {quote}{dollar}@{quote})")
            );
            format!("{args}\n{}", self.block.translate(meta))
        }
    }
}

impl DocumentationModule for Main {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
