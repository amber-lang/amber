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
    pub expr: Box<Expr>,
    pub is_expr: bool,
    pub is_unsafe: bool,
    pub is_silent: bool
}

pub struct CommandModifierExpr {
    pub modifier: CommandModifier
}

impl Typed for CommandModifierExpr {
    fn get_type(&self) -> Type {
        self.modifier.expr.get_type()
    }
}

impl CommandModifier {
    pub fn parse_expr(mut self) -> Self {
        self.is_expr = true;
        self
    }
}

impl SyntaxModule<ParserMetadata> for CommandModifier {
    syntax_name!("Command Modifier");

    fn new() -> Self {
        CommandModifier {
            block: Box::new(Block::new()),
            expr: Box::new(Expr::new()),
            is_expr: false,
            is_unsafe: false,
            is_silent: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let mut is_matched = false;
        let mut sequence = String::new();
        let tok = meta.get_current_token();
        loop {
            match meta.get_current_token() {
                Some(tok) => {
                    sequence.push_str(tok.word.as_str());
                    sequence.push_str(" ");
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
        sequence = sequence.trim().to_string();
        swap(&mut self.is_unsafe, &mut meta.context.is_unsafe_ctx);
        if self.is_expr {
            syntax(meta, &mut *self.expr)?;
            if !matches!(self.expr.value, Some(ExprType::CommandExpr(_))) {
                return error!(meta, tok, format!("Expected command expression, after '{sequence}' command modifiers."));
            }
        } else {
            match token(meta, "{") {
                Ok(_) => {
                    syntax(meta, &mut *self.block)?;
                    token(meta, "}")?;
                },
                Err(_) => {
                    let mut statement = Statement::new();
                    syntax(meta, &mut statement)?;
                    self.block.push_statement(statement);
                }
            }
        }
        swap(&mut self.is_unsafe, &mut meta.context.is_unsafe_ctx);
        Ok(())
    }
}

impl TranslateModule for CommandModifier {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        meta.silenced = true;
        let result = if self.is_expr {
            return self.expr.translate(meta)
        } else {
            self.block.translate(meta)
        };
        meta.silenced = false;
        result
    }
}