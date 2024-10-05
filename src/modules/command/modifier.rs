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
    pub is_trust: bool,
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
        let mut is_trust_holder = self.is_trust;
        if self.is_trust {
            swap(&mut is_trust_holder, &mut meta.context.is_trust_ctx);
        }
        let result = context(self, meta);
        // Swap back the value
        if self.is_trust {
            swap(&mut is_trust_holder, &mut meta.context.is_trust_ctx);
        }
        result
    }

    fn parse_modifier_sequence(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        loop {
            match meta.get_current_token() {
                Some(tok) => {
                    match tok.word.as_str() {
                        trust @ ("trust" | "unsafe") => {
                            if trust == "unsafe" {
                                let message = Message::new_warn_at_token(meta, Some(tok.clone()))
                                .message("The keyword `unsafe` has been deprecated in favor of `trust`.")
                                .comment("Learn more about this change: https://docs.amber-lang.com/basic_syntax/commands#command-modifiers");
                                meta.add_message(message);
                            }
                            if self.is_trust {
                                return error!(meta, Some(tok.clone()), "You already declared `trust` modifier before");
                            }
                            self.is_trust = true;
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
            is_trust: false,
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
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
