use heraclitus_compiler::prelude::*;
use heraclitus_compiler::compiling::failing::position_info::PositionInfo;
use crate::{fragments, raw_fragment};
use crate::modules::prelude::*;
use crate::modules::block::Block;
use crate::modules::types::Type;
use crate::modules::variable::variable_name_extensions;
use crate::utils::context::{VariableDecl, VariableDeclWarn};
use crate::utils::metadata::ParserMetadata;

#[derive(Debug, Clone, PartialEq)]
pub enum FailureType {
    Failed,
    Succeeded,
    Exited,
}

impl FailureType {
    pub fn to_string(&self) -> &'static str {
        match self {
            FailureType::Failed => "failed",
            FailureType::Succeeded => "succeeded",
            FailureType::Exited => "exited",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FailureHandler {
    pub is_parsed: bool,
    pub failure_type: FailureType,
    pub is_question_mark: bool,
    error_position: Option<PositionInfo>,
    function_name: Option<String>,
    is_main: bool,
    block: Box<Block>,
    param_name: String,
    param_name_tok: Option<Token>,
    param_global_id: Option<usize>
}

impl FailureHandler {
    pub fn set_position(&mut self, position: PositionInfo) {
        self.error_position = Some(position);
    }

    pub fn set_function_name(&mut self, name: String) {
        self.function_name = Some(name);
    }
}

impl SyntaxModule<ParserMetadata> for FailureHandler {
    syntax_name!("Failure Handler Expression");

    fn new() -> Self {
        FailureHandler {
            is_parsed: false,
            failure_type: FailureType::Failed,
            is_question_mark: false,
            is_main: false,
            function_name: None,
            error_position: None,
            block: Box::new(Block::new().with_needs_noop().with_condition()),
            param_name: String::new(),
            param_name_tok: None,
            param_global_id: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();

        // Check for ? operator first
        if token(meta, "?").is_ok() {
            if !meta.context.is_fun_ctx && !meta.context.is_main_ctx {
                return error!(meta, tok, "The '?' operator can only be used in the main block or inside a function body")
            }
            self.is_question_mark = true;
            self.failure_type = FailureType::Failed;
        } else {
            let keyword = ["failed", "succeeded", "exited"].iter().fold(None, |acc, keyword| {
                acc.or_else(|| token(meta, keyword).ok().map(|_| *keyword))
            });

            match keyword {
                Some("failed") => {
                    self.failure_type = FailureType::Failed;

                    // Check if there's a parameter in parentheses (only for failed)
                    if token(meta, "(").is_ok() {
                        context!({
                            let param_tok = meta.get_current_token();

                            // Check if we immediately hit a closing paren (empty parameter)
                            if token(meta, ")").is_ok() {
                                let pos = PositionInfo::from_between_tokens(meta, param_tok, meta.get_current_token());
                                return error_pos!(meta, pos, "Parameter name cannot be empty");
                            }

                            self.param_name_tok = meta.get_current_token();
                            self.param_name = variable(meta, variable_name_extensions())?;
                            token(meta, ")")?;

                            // Parse the block (scope and variable will be added in typecheck)
                            syntax(meta, &mut *self.block)?;
                            Ok(())
                        }, |pos| {
                            error_pos!(meta, pos, "Failed to parse failed block")
                        })?;
                    } else {
                        // No parameter, parse block normally
                        syntax(meta, &mut *self.block)?;
                    }
                },
                Some("succeeded") => {
                    self.failure_type = FailureType::Succeeded;
                    syntax(meta, &mut *self.block)?;
                },
                Some("exited") => {
                    self.failure_type = FailureType::Exited;

                    // Check if there's a parameter in parentheses (optional for exited)
                    if token(meta, "(").is_ok() {
                        context!({
                            let param_tok = meta.get_current_token();

                            // Check if we immediately hit a closing paren (empty parameter)
                            if token(meta, ")").is_ok() {
                                let pos = PositionInfo::from_between_tokens(meta, param_tok, meta.get_current_token());
                                return error_pos!(meta, pos, "Parameter name cannot be empty");
                            }

                            self.param_name_tok = meta.get_current_token();
                            self.param_name = variable(meta, variable_name_extensions())?;
                            token(meta, ")")?;

                            // Parse the block (scope and variable will be added in typecheck)
                            syntax(meta, &mut *self.block)?;
                            Ok(())
                        }, |pos| {
                            error_pos!(meta, pos, "Failed to parse exited block")
                        })?;
                    } else {
                        // No parameter, parse block normally
                        syntax(meta, &mut *self.block)?;
                    }
                },
                Some(keyword) => {
                    unimplemented!("Keyword '{keyword}' is not yet implemented")
                },
                None => {
                    if meta.context.is_trust_ctx {
                        self.is_main = meta.context.is_main_ctx;
                        self.is_parsed = true;
                        return Ok(());
                    } else {
                        return Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
                    }
                }
            }

            // Check for empty block once after parsing the appropriate variant
            if self.block.is_empty() {
                let message = Message::new_warn_at_token(meta, meta.get_current_token())
                    .message(format!("Empty {} block", self.failure_type.to_string()))
                    .comment("You should use 'trust' modifier to run commands without handling errors");
                meta.add_message(message);
            }

            if let Some(keyword) = keyword {
                let next_tok = meta.get_current_token();
                let next_word = token_by(meta, |word| ["failed", "succeeded", "exited"].contains(&word.as_str()));
                if let Ok(word) = next_word {
                    return error!(meta, next_tok => {
                        message: format!("Cannot use both '{keyword}' and '{word}' blocks for the same command"),
                        comment: format!("Use either '{keyword}' to handle both success and failure, 'failed' or 'succeeded' blocks, but not both")
                    });
                }
            }
        }


        self.is_main = meta.context.is_main_ctx;
        self.is_parsed = true;
        Ok(())
    }
}

impl TypeCheckModule for FailureHandler {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // If we have a parameter (exit code for failed or exited), add it to scope and typecheck the block
        if !self.param_name.is_empty() && (self.failure_type == FailureType::Failed || self.failure_type == FailureType::Exited) {
            meta.with_push_scope(true, |meta| {
                let var = VariableDecl::new(self.param_name.clone(), Type::Num)
                    .with_warn(VariableDeclWarn::from_token(meta, self.param_name_tok.clone()));
                self.param_global_id = meta.add_var(var);
                self.block.typecheck(meta)
            })
        } else {
            self.block.typecheck(meta)
        }
    }
}

impl TranslateModule for FailureHandler {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if !self.is_parsed {
            return FragmentKind::Empty;
        }

        let is_expr_ctx = meta.expr_ctx;
        meta.expr_ctx = false;
        let block = self.block.translate(meta);
        meta.expr_ctx = is_expr_ctx;
        // the condition of '$?' clears the status code thus we need to store it in a variable
        let status_variable_stmt = VarStmtFragment::new("__status", Type::Int, fragments!("$?"));
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
            ], false).to_frag();
        }

        match &block {
            FragmentKind::Empty => {
                status_variable_stmt.to_frag()
            },
            FragmentKind::Block(block) if block.statements.is_empty() => {
                status_variable_stmt.to_frag()
            },
            _ => {
                match self.failure_type {
                    FailureType::Failed => {
                        // If a parameter name is provided, assign the status to it
                        if !self.param_name.is_empty() {
                            let param_assignment = VarStmtFragment::new(&self.param_name, Type::Int, status_variable_expr.clone().to_frag())
                                .with_global_id(self.param_global_id);

                            BlockFragment::new(vec![
                                status_variable_stmt.to_frag(),
                                fragments!("if [ ", status_variable_expr.to_frag(), " != 0 ]; then"),
                                param_assignment.to_frag(),
                                block,
                                fragments!("fi"),
                            ], false).to_frag()
                        } else {
                            BlockFragment::new(vec![
                                status_variable_stmt.to_frag(),
                                fragments!("if [ ", status_variable_expr.to_frag(), " != 0 ]; then"),
                                block,
                                fragments!("fi"),
                            ], false).to_frag()
                        }
                    },
                    FailureType::Succeeded => {
                        BlockFragment::new(vec![
                            status_variable_stmt.to_frag(),
                            fragments!("if [ ", status_variable_expr.to_frag(), " = 0 ]; then"),
                            block,
                            fragments!("fi"),
                        ], false).to_frag()
                    },
                    FailureType::Exited => {
                        // Exited always runs, regardless of exit code
                        // If a parameter name is provided, assign the status to it
                        if !self.param_name.is_empty() {
                            let param_assignment = VarStmtFragment::new(&self.param_name, Type::Int, status_variable_expr.clone().to_frag())
                                .with_global_id(self.param_global_id);

                            BlockFragment::new(vec![
                                status_variable_stmt.to_frag(),
                                param_assignment.to_frag(),
                                block,
                            ], false).to_frag()
                        } else {
                            BlockFragment::new(vec![
                                status_variable_stmt.to_frag(),
                                block,
                            ], false).to_frag()
                        }
                    }
                }
            }
        }
    }
}
