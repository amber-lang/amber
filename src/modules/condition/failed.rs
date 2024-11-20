use heraclitus_compiler::prelude::*;
use crate::modules::block::Block;
use crate::modules::statement::stmt::Statement;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Failed {
    pub is_parsed: bool,
    is_question_mark: bool,
    is_main: bool,
    block: Box<Block>
}

impl SyntaxModule<ParserMetadata> for Failed {
    syntax_name!("Failed Expression");

    fn new() -> Self {
        Failed {
            is_parsed: false,
            is_question_mark: false,
            is_main: false,
            block: Box::new(Block::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        if token(meta, "?").is_ok() {
            if !meta.context.is_fun_ctx && !meta.context.is_main_ctx && !meta.context.is_trust_ctx {
                return error!(meta, tok, "The '?' operator can only be used in the main block or function body")
            }
            self.is_question_mark = true;
        } else {
            match token(meta, "failed") {
                Ok(_) => match token(meta, "{") {
                    Ok(_) => {
                        let tok = meta.get_current_token();
                        syntax(meta, &mut *self.block)?;
                        if self.block.is_empty() {
                            let message = Message::new_warn_at_token(meta, tok)
                                .message("Empty failed block")
                                .comment("You should use 'trust' modifier to run commands without handling errors");
                            meta.add_message(message);
                        }
                        token(meta, "}")?;
                    },
                    Err(_) => {
                        match token(meta, ":") {
                            Ok(_) => {
                                let mut statement = Statement::new();
                                syntax(meta, &mut statement)?;
                                self.block.push_statement(statement);
                            },
                            Err(_) => return error!(meta, tok, "Failed expression must be followed by a block or statement")
                        }
                    }
                },
                Err(_) => if meta.context.is_trust_ctx {
                    self.is_main = meta.context.is_main_ctx;
                    self.is_parsed = true;
                    return Ok(());
                } else {
                    return error!(meta, tok, "Failed expression must be followed by a block or statement")
                }
            }
        }
        self.is_main = meta.context.is_main_ctx;
        self.is_parsed = true;
        Ok(())
    }
}

impl TranslateModule for Failed {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        if self.is_parsed {
            let block = self.block.translate(meta);
            let ret = if self.is_main { "exit $__AS" } else { "return $__AS" };
            // the condition of '$?' clears the status code thus we need to store it in a variable
            if self.is_question_mark {
                // if the failed expression is in the main block we need to clear the return value
                let clear_return = if !self.is_main {
                    let fun_meta = meta.fun_meta.as_ref().expect("Function name and return type not set");
                    format!("{}={}", fun_meta.mangled_name(), fun_meta.default_return())
                } else {
                    String::new()
                };
                [
                    "__AS=$?;",
                    "if [ $__AS != 0 ]; then",
                    &clear_return,
                    ret,
                    "fi",
                ].join("\n")
            } else if &block == ":" {
                "__AS=$?".into()
            } else {
                [
                    "__AS=$?;",
                    "if [ $__AS != 0 ]; then",
                    &block,
                    "fi",
                ].join("\n")
            }
        } else {
            String::new()
        }
    }
}
