use super::modifier::CommandModifier;
use crate::fragments;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::interpolated_region::{
    parse_interpolated_region, InterpolatedRegionType,
};
use crate::modules::expression::literal::text::TextPart;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Command {
    parts: Vec<TextPart>,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
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
            parts: vec![],
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            let tok = meta.get_current_token();
            self.parts = parse_interpolated_region(meta, &InterpolatedRegionType::Command)?;

            // Set position for failure handler
            let position = PositionInfo::from_between_tokens(meta, tok.clone(), meta.get_current_token());
            self.failure_handler.set_position(position.clone());

            // Try to parse failure handler (failed, succeeded, or exited)
            match syntax(meta, &mut self.failure_handler) {
                Ok(_) => Ok(()),
                Err(Failure::Quiet(_)) => {
                    // No failure handler found
                    error!(meta, tok => {
                        message: "The command can potentially fail. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
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
        self.modifier.use_modifiers(meta, |modifier, meta| {
            for part in self.parts.iter_mut() {
                part.typecheck(meta)?;
            }
            if modifier.is_trust && self.failure_handler.is_question_mark {
                let tok = meta.get_current_token();
                return error!(meta, tok, "The '?' operator cannot be used with the 'trust' modifier because 'trust' ignores failure while '?' propagates it");
            }

            self.failure_handler.typecheck(meta)
        })
    }
}

impl TranslateModule for Command {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let is_silenced = self.modifier.is_silent || meta.silenced;
        let is_suppress = self.modifier.is_suppress || meta.suppress;
        let is_sudoed = self.modifier.is_sudo || meta.sudoed;

        let translation = meta.with_silenced(is_silenced, |meta| {
            meta.with_suppress(is_suppress, |meta| {
                meta.with_sudoed(is_sudoed, |meta| {
                    let parts = TextPart::to_interpolable_parts(&self.parts, meta);
                    InterpolableFragment::new(parts, InterpolableRenderType::GlobalContext)
                        .to_frag()
                })
            })
        });

        let silent = meta.with_silenced(is_silenced, |meta| meta.gen_silent().to_frag());
        let suppress = meta.with_suppress(is_suppress, |meta| meta.gen_suppress().to_frag());
        let sudo_prefix = meta.with_sudoed(is_sudoed, |meta| meta.gen_sudo_prefix().to_frag());
        let translation = fragments!(sudo_prefix, translation, suppress, silent);

        let handler = self.failure_handler.translate(meta);
        let is_statement = !meta.expr_ctx;
        let has_failure_handler = self.failure_handler.is_parsed;

        match (is_statement, has_failure_handler) {
            (true, true) => {
                meta.stmt_queue.push_back(translation);
                handler
            }
            (true, false) => translation,
            (false, false) => SubprocessFragment::new(translation).to_frag(),
            (false, true) => {
                let id = meta.gen_value_id();
                let value = SubprocessFragment::new(translation).to_frag();
                let var_stmt =
                    VarStmtFragment::new("command", Type::Text, value).with_global_id(id);
                let var_expr = meta.push_ephemeral_variable(var_stmt);
                meta.stmt_queue.push_back(handler);
                var_expr.to_frag()
            }
        }
    }
}

impl DocumentationModule for Command {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
