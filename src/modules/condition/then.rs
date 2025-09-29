use heraclitus_compiler::prelude::*;
use crate::fragments;
use crate::modules::prelude::*;
use crate::modules::block::Block;
use crate::modules::types::Type;

#[derive(Debug, Clone)]
pub struct Then {
    pub is_parsed: bool,
    error_position: Option<PositionInfo>,
    function_name: Option<String>,
    block: Box<Block>,
    param_name: String
}

impl Then {
    pub fn set_position(&mut self, position: PositionInfo) {
        self.error_position = Some(position);
    }

    pub fn set_function_name(&mut self, name: String) {
        self.function_name = Some(name);
    }
}

impl SyntaxModule<ParserMetadata> for Then {
    syntax_name!("Then Expression");

    fn new() -> Self {
        Then {
            is_parsed: false,
            function_name: None,
            error_position: None,
            block: Box::new(Block::new().with_needs_noop().with_condition()),
            param_name: String::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        match token(meta, "then") {
            Ok(_) => {
                let tok = meta.get_current_token();
                
                // Parse the parameter in parentheses
                token(meta, "(")?;
                let param_tok = meta.get_current_token();
                let param_name = token_by(meta, |word| word.chars().all(|c| c.is_alphanumeric() || c == '_'))?;
                
                if param_name.is_empty() {
                    return error!(meta, param_tok, "Parameter name cannot be empty");
                }
                
                self.param_name = param_name;
                token(meta, ")")?;
                
                syntax(meta, &mut *self.block)?;
                if self.block.is_empty() {
                    let message = Message::new_warn_at_token(meta, tok)
                        .message("Empty then block")
                        .comment("You should use 'trust' modifier to run commands without handling errors");
                    meta.add_message(message);
                }
                self.is_parsed = true;
                Ok(())
            },
            Err(_) => {
                // If we're in a trust context, mark as parsed
                if meta.context.is_trust_ctx {
                    self.is_parsed = true;
                }
                // Otherwise, return quietly (no then block found)
                Ok(())
            }
        }
    }
}

impl TranslateModule for Then {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.is_parsed {
            let block = self.block.translate(meta);
            // Store the exit status in a variable and pass it to the then block
            let status_variable_stmt = VarStmtFragment::new("__status", Type::Num, fragments!("$?"));
            let param_variable_stmt = VarStmtFragment::new(&self.param_name, Type::Num, VarExprFragment::from_stmt(&status_variable_stmt).to_frag());
            
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
                        param_variable_stmt.to_frag(),
                        block,
                    ], false).to_frag()
                }
            }
        } else {
            FragmentKind::Empty
        }
    }
}