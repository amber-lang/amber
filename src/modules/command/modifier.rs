use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::block::Block;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct CommandModifier {
    pub block: Box<Block>,
    pub is_block: bool,
    pub is_unsafe: bool,
    pub is_silent: bool
}

impl CommandModifier {
    pub fn parse_expr(mut self) -> Self {
        self.is_block = false;
        self
    }

    pub fn use_modifiers<F>(
        &mut self, meta: &mut ParserMetadata, context: F
    ) -> SyntaxResult where F: FnOnce(&mut Self, &mut ParserMetadata) -> SyntaxResult {
        let mut is_unsafe_holder = self.is_unsafe;
        if self.is_unsafe {
            swap(&mut is_unsafe_holder, &mut meta.context.is_unsafe_ctx);
        }
        let result = context(self, meta);
        // Swap back the value
        if self.is_unsafe {
            swap(&mut is_unsafe_holder, &mut meta.context.is_unsafe_ctx);
        }
        result
    }

    fn parse_modifier_sequence(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        loop {
            match meta.get_current_token() {
                Some(tok) => {
                    match tok.word.as_str() {
                        "unsafe" => {
                            if self.is_unsafe {
                                return error!(meta, Some(tok.clone()), "You already declared `unsafe` modifier before");
                            }
                            self.is_unsafe = true;
                            meta.increment_index();
                        },
                        "silent" => {
                            if self.is_silent {
                                return error!(meta, Some(tok.clone()), "You already declared `silent` modifier before");
                            }
                            self.is_silent = true;
                            meta.increment_index();
                        },
                        _ => break
                    }
                },
                None => return Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
            }
        }
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for CommandModifier {
    syntax_name!("Command Modifier");

    fn new() -> Self {
        CommandModifier {
            block: Box::new(Block::new()),
            is_block: true,
            is_unsafe: false,
            is_silent: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.parse_modifier_sequence(meta)?;
        if self.is_block {
            return self.use_modifiers(meta, |this, meta| {
                token(meta, "{")?;
                syntax(meta, &mut *this.block)?;
                token(meta, "}")?;
                Ok(())
            })
        }
        Ok(())
    }
}

impl TranslateModule for CommandModifier {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        if self.is_block {
            meta.silenced = self.is_silent;
            let result = self.block.translate(meta);
            meta.silenced = false;
            result
        } else {
            String::new()
        }
    }
}

impl DocumentationModule for CommandModifier {
    fn document(&self) -> String {
        "".to_string()
    }
}

