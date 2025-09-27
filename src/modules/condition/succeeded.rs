use heraclitus_compiler::prelude::*;
use crate::fragments;
use crate::modules::prelude::*;
use crate::modules::block::Block;
use crate::modules::types::Type;

#[derive(Debug, Clone)]
pub struct Succeeded {
    pub is_parsed: bool,
    error_position: Option<PositionInfo>,
    function_name: Option<String>,
    is_main: bool,
    block: Box<Block>
}

impl Succeeded {
    pub fn set_position(&mut self, position: PositionInfo) {
        self.error_position = Some(position);
    }

    pub fn set_function_name(&mut self, name: String) {
        self.function_name = Some(name);
    }
}

impl SyntaxModule<ParserMetadata> for Succeeded {
    syntax_name!("Succeeded Expression");

    fn new() -> Self {
        Succeeded {
            is_parsed: false,
            is_main: false,
            function_name: None,
            error_position: None,
            block: Box::new(Block::new().with_needs_noop().with_condition())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        match token(meta, "succeeded") {
            Ok(_) => {
                let tok = meta.get_current_token();
                syntax(meta, &mut *self.block)?;
                if self.block.is_empty() {
                    let message = Message::new_warn_at_token(meta, tok)
                        .message("Empty succeeded block")
                        .comment("You should use 'trust' modifier to run commands without handling errors");
                    meta.add_message(message);
                }
                self.is_main = meta.context.is_main_ctx;
                self.is_parsed = true;
                Ok(())
            },
            Err(_) => {
                // If we're in a trust context, mark as parsed
                if meta.context.is_trust_ctx {
                    self.is_main = meta.context.is_main_ctx;
                    self.is_parsed = true;
                }
                // Otherwise, return quietly (no succeeded block found)
                Ok(())
            }
        }
    }
}

impl TranslateModule for Succeeded {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.is_parsed {
            let block = self.block.translate(meta);
            // the condition of '$?' clears the status code thus we need to store it in a variable
            let status_variable_stmt = VarStmtFragment::new("__status", Type::Num, fragments!("$?"));
            let status_variable_expr = VarExprFragment::from_stmt(&status_variable_stmt);
            
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
                        fragments!("if [ ", status_variable_expr.to_frag(), " = 0 ]; then"),
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