use heraclitus_compiler::prelude::*;
use crate::fragments;
use crate::modules::prelude::*;
use crate::modules::block::Block;
use crate::modules::statement::statement::Statement;

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
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        if self.is_parsed {
            let block = self.block.translate(meta);
            let ret = if self.is_main { "exit $__status" } else { "return $__status" };
            // the condition of '$?' clears the status code thus we need to store it in a variable
            if self.is_question_mark {
                // Set default return value if failure happened in a function
                let clear_return = if !self.is_main {
                    let fun_meta = meta.fun_meta.as_ref().expect("Function name and return type not set");
                    let statement = format!("{}={}", fun_meta.mangled_name(), fun_meta.default_return());
                    RawFragment::new(&statement).to_frag()
                } else {
                    TranslationFragment::Empty
                };
                let ret = RawFragment::new(&ret).to_frag();
                return BlockFragment::new(vec![
                    fragments!("__status=$?;"),
                    fragments!("if [ $__status != 0 ]; then"),
                    BlockFragment::new(vec![
                        clear_return,
                        ret,
                    ], true).to_frag(),
                    fragments!("fi"),
                ], false).to_frag()
            }
            match &block {
                TranslationFragment::Empty => {
                    return fragments!("__status=$?")
                },
                TranslationFragment::Block(block) if block.is_empty() => {
                    return fragments!("__status=$?")
                },
                _ => {}
            }
            BlockFragment::new(vec![
                fragments!("__status=$?"),
                fragments!("if [ $__status != 0 ]; then"),
                block,
                fragments!("fi"),
            ], false).to_frag()
        } else {
            TranslationFragment::Empty
        }
    }
}
