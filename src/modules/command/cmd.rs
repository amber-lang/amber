use std::mem::swap;
use crate::modules::types::{Type, Typed};
use crate::modules::expression::literal::bool;
use crate::modules::condition::failed::Failed;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::literal::parse_interpolated_region;
use super::modifier::CommandModifier;
use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;

#[derive(Debug, Clone)]
pub struct Command {
    strings: Vec<String>,
    interps: Vec<Expr>,
    modifier: CommandModifier,
    failed: Failed
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
            failed: Failed::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            let tok = meta.get_current_token();
            (self.strings, self.interps) = parse_interpolated_region(meta, '$')?;
            match syntax(meta, &mut self.failed) {
                Ok(_) => Ok(()),
                Err(Failure::Quiet(_)) => error!(meta, tok => {
                    message: "Every command statement must handle failed execution",
                    comment: "You can use '?' in the end to propagate the failure"
                }),
                Err(err) => Err(err)
            }
        })
    }
}

impl Command {
    fn translate_command(&self, meta: &mut TranslateMetadata, is_statement: bool) -> TranslationFragment {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta).unquote())
            .collect::<Vec<TranslationFragment>>();
        let failed = self.failed.translate(meta);

        let mut is_silent = self.modifier.is_silent || meta.silenced;
        swap(&mut is_silent, &mut meta.silenced);

        let translation = InterpolableFragment::new(
            self.strings.clone(),
            interps,
            InterpolableRenderType::GlobalContext
        ).to_frag();

        let silent = meta.gen_silent().to_frag();
        let translation = fragments!(translation, silent);
        swap(&mut is_silent, &mut meta.silenced);

        if is_statement {
            return if let TranslationFragment::Empty = failed { translation } else {
                meta.stmt_queue.push_back(translation);
                failed
            }
        }

        if let TranslationFragment::Empty = failed {
            SubprocessFragment::new(translation).to_frag()
        } else {
            let id = meta.gen_value_id();
            let value = SubprocessFragment::new(translation).to_frag();
            let variable = meta.push_stmt_variable("__command", Some(id), Type::Text, value);
            meta.stmt_queue.push_back(failed);
            variable.to_frag()
        }
    }

    pub fn translate_command_statement(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        self.translate_command(meta, true)
    }
}

impl TranslateModule for Command {
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        self.translate_command(meta, false)
    }
}

impl DocumentationModule for Command {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
