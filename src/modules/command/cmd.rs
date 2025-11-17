use std::mem::swap;
use crate::modules::types::{Type, Typed};
use crate::modules::expression::literal::bool;
use crate::modules::condition::failure_handler::FailureHandler;
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
    failure_handler: FailureHandler
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
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            let tok = meta.get_current_token();
            (self.strings, self.interps) = parse_interpolated_region(meta, &InterpolatedRegionType::Command)?;

            // Set position for failure handler
            let position = PositionInfo::from_between_tokens(meta, tok.clone(), meta.get_current_token());
            self.failure_handler.set_position(position.clone());

            // Try to parse failure handler (failed, succeeded, or exited)
            match syntax(meta, &mut self.failure_handler) {
                Ok(_) => Ok(()),
                Err(Failure::Quiet(_)) => {
                    // No failure handler found
                    error!(meta, tok => {
                        message: "Every command statement must handle execution result",
                        comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both, or 'trust' modifier to ignore results"
                    })
                },
                Err(err) => Err(err)
            }
        })
    }
}

impl TypeCheckModule for Command {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        for interp in self.interps.iter_mut() {
            interp.typecheck(meta)?;
        }
        self.failure_handler.typecheck(meta)
    }
}

impl Command {
    fn translate_command(&self, meta: &mut TranslateMetadata, is_statement: bool) -> FragmentKind {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta).with_quotes(false))
            .collect::<Vec<FragmentKind>>();
        let handler = self.failure_handler.translate(meta);

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

        if is_statement {
            if self.failure_handler.is_parsed {
                meta.stmt_queue.push_back(translation);
                handler
            } else {
                translation
            }
        } else if !self.failure_handler.is_parsed {
            SubprocessFragment::new(translation).to_frag()
        } else {
            let id = meta.gen_value_id();
            let value = SubprocessFragment::new(translation).to_frag();
            let var_stmt = VarStmtFragment::new("command", Type::Text, value).with_global_id(id);
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
