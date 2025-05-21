use heraclitus_compiler::prelude::*;
use crate::{fragments, raw_fragment};
use crate::modules::prelude::*;
use crate::modules::block::Block;
use crate::modules::statement::stmt::Statement;
use crate::modules::types::Type;

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
            block: Box::new(Block::new().with_needs_noop().with_condition())
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
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.is_parsed {
            let block = self.block.translate(meta);
            // the condition of '$?' clears the status code thus we need to store it in a variable
            let status_variable_stmt = VarStmtFragment::new("__status", Type::Num, fragments!("$?"));
            let status_variable_expr = VarExprFragment::from_stmt(&status_variable_stmt);
            if self.is_question_mark {
                // Set default return value if failure happened in a function
                let clear_return = if !self.is_main {
                    let fun_meta = meta.fun_meta.as_ref().expect("Function name and return type not set");
                    let stmt = VarStmtFragment::new(&fun_meta.mangled_name(), fun_meta.get_type(), fun_meta.default_return())
                        .with_optimization_when_unused(false);
                    stmt.to_frag()
                } else {
                    FragmentKind::Empty
                };
                let ret = if self.is_main { "exit" } else { "return" };
                let ret = fragments!(raw_fragment!("{ret} "), status_variable_expr.clone().to_frag());
                return BlockFragment::new(vec![
                    status_variable_stmt.to_frag(),
                    fragments!("if [ ", status_variable_expr.to_frag(), " != 0 ]; then"),
                    BlockFragment::new(vec![
                        clear_return,
                        ret,
                    ], true).to_frag(),
                    fragments!("fi"),
                ], false).to_frag()
            }
            match &block {
                FragmentKind::Empty => {
                    status_variable_stmt.to_frag()
                },
                FragmentKind::Block(block) if block.statements.is_empty() => {
                    status_variable_stmt.to_frag()
                },
                _ => {
                    BlockFragment::new(vec![
                        status_variable_stmt.to_frag(),
                        fragments!("if [ ", status_variable_expr.to_frag(), " != 0 ]; then"),
                        block,
                        fragments!("fi"),
                    ], false).to_frag()
                }
            }
        } else {
            FragmentKind::Empty
        }
    }
}
