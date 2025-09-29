use std::mem::swap;
use crate::modules::types::{Type, Typed};
use crate::modules::expression::literal::bool;
use crate::modules::condition::failed::Failed;
use crate::modules::condition::succeeded::Succeeded;
use crate::modules::condition::then::Then;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::interpolated_region::{InterpolatedRegionType, parse_interpolated_region};
use super::modifier::CommandModifier;
use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;

#[derive(Debug, Clone)]
pub struct Command {
    strings: Vec<String>,
    interps: Vec<Expr>,
    modifier: CommandModifier,
    failed: Failed,
    succeeded: Succeeded,
    then: Then
}

impl Typed for Command {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Command {
    syntax_name!("Command");

    fn new() -> Self {
        Command {
            strings: vec![],
            interps: vec![],
            modifier: CommandModifier::new().parse_expr(),
            failed: Failed::new(),
            succeeded: Succeeded::new(),
            then: Then::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            let tok = meta.get_current_token();
            (self.strings, self.interps) = parse_interpolated_region(meta, &InterpolatedRegionType::Command)?;
            
            // Set position for failed and succeeded handlers
            let position = PositionInfo::from_between_tokens(meta, tok.clone(), meta.get_current_token());
            self.failed.set_position(position.clone());
            self.succeeded.set_position(position);
            
            // Try to parse then block first
            syntax(meta, &mut self.then)?;
            
            if self.then.is_parsed {
                // Check for conflicts with failed or succeeded blocks
                if let Ok(word) = token_by(meta, |word| word == "failed" || word == "succeeded") {
                    return error!(meta, meta.get_current_token() => {
                        message: format!("Cannot use both 'then' and '{}' blocks for the same command", word),
                        comment: "Use either 'then' to handle both success and failure, or 'failed'/'succeeded' blocks, but not both"
                    });
                }
                // When then block is used, no trust modifier is required
                return Ok(());
            }
            
            // Try to parse succeeded block
            syntax(meta, &mut self.succeeded)?;
            
            // If succeeded block was parsed successfully, check for conflicts with failed
            if self.succeeded.is_parsed {
                // Check if there's an attempt to use failed block as well
                if token(meta, "failed").is_ok() {
                    return error!(meta, meta.get_current_token() => {
                        message: "Cannot use both 'succeeded' and 'failed' blocks for the same command",
                        comment: "Use either 'succeeded' or 'failed' block, but not both"
                    });
                }
                return Ok(());
            }

            // If no succeeded block, try to parse failed block
            match syntax(meta, &mut self.failed) {
                Ok(_) => Ok(()),
                Err(Failure::Quiet(_)) => {
                    // Neither succeeded, failed, nor then block found
                    error!(meta, tok => {
                        message: "Every command statement must handle execution result", 
                        comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'then' block to handle both, or 'trust' modifier to ignore results"
                    })
                },
                Err(err) => Err(err)
            }
        })
    }
}

impl Command {
    fn translate_command(&self, meta: &mut TranslateMetadata, is_statement: bool) -> FragmentKind {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta).with_quotes(false))
            .collect::<Vec<FragmentKind>>();
        let failed = self.failed.translate(meta);
        let succeeded = self.succeeded.translate(meta);
        let then = self.then.translate(meta);

        let mut is_silent = self.modifier.is_silent || meta.silenced;
        let mut is_sudo = self.modifier.is_sudo || meta.sudoed;
        swap(&mut is_silent, &mut meta.silenced);
        swap(&mut is_sudo, &mut meta.sudoed);

        let translation = InterpolableFragment::new(
            self.strings.clone(),
            interps,
            InterpolableRenderType::GlobalContext
        ).to_frag();

        let silent = meta.gen_silent().to_frag();
        let sudo_prefix = meta.gen_sudo_prefix().to_frag();
        let translation = ListFragment::new(vec![sudo_prefix, translation, silent])
            .with_spaces()
            .to_frag();
        swap(&mut is_silent, &mut meta.silenced);
        swap(&mut is_sudo, &mut meta.sudoed);

        // Choose between failed, succeeded, then, or no handler
        let handler = if self.then.is_parsed {
            then
        } else if self.failed.is_parsed {
            failed
        } else if self.succeeded.is_parsed {
            succeeded
        } else {
            FragmentKind::Empty
        };

        if is_statement {
            if let FragmentKind::Empty = handler {
                translation
            } else {
                meta.stmt_queue.push_back(translation);
                handler
            }
        } else if let FragmentKind::Empty = handler {
            SubprocessFragment::new(translation).to_frag()
        } else {
            let id = meta.gen_value_id();
            let value = SubprocessFragment::new(translation).to_frag();
            let var_stmt = VarStmtFragment::new("__command", Type::Text, value).with_global_id(id);
            let var_expr = meta.push_ephemeral_variable(var_stmt);
            meta.stmt_queue.push_back(handler);
            var_expr.to_frag()
        }
    }

    pub fn translate_command_statement(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        self.translate_command(meta, true)
    }
}

impl TranslateModule for Command {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        self.translate_command(meta, false)
    }
}

impl DocumentationModule for Command {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
