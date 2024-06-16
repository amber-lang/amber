use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::modules::block::Block;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::statement::stmt::Statement;
use crate::modules::types::{Typed, Type};
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
        self.is_block = true;
        self
    }

    fn flip_modifiers(&mut self, meta: &mut ParserMetadata, is_unsafe: bool) {
        if is_unsafe {
            swap(&mut self.is_unsafe, &mut meta.context.is_unsafe_ctx);
        }
    }

    fn parse_modifier_sequence(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let mut is_matched = false;
        loop {
            match meta.get_current_token() {
                Some(tok) => {
                    match tok.word.as_str() {
                        "unsafe" => {
                            self.is_unsafe = true;
                            is_matched = true;
                            meta.increment_index();
                        },
                        "silent" => {
                            self.is_silent = true;
                            is_matched = true;
                            meta.increment_index();
                        },
                        _ => if is_matched {
                            break;
                        } else {
                            return Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
                        }
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
        self.parse_modifier_sequence(meta);
        if self.is_block {
            let is_unsafe = self.is_unsafe;
            self.flip_modifiers(meta, is_unsafe);
            token(meta, "{")?;
            if let Err(err) = syntax(meta, &mut *self.block) {
                self.flip_modifiers(meta, is_unsafe);
                return Err(err)
            }
            token(meta, "}")?;
            self.flip_modifiers(meta, is_unsafe);
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
