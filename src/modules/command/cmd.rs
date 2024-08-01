use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::{condition::failed::Failed, expression::literal::bool, types::{Type, Typed}}, utils::{ParserMetadata, TranslateMetadata}};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::modules::expression::literal::{parse_interpolated_region, translate_interpolated_region};
use super::modifier::CommandModifier;

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
    fn translate_command(&self, meta: &mut TranslateMetadata, is_statement: bool) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        let failed = self.failed.translate(meta);
        let mut is_silent = self.modifier.is_silent || meta.silenced;
        swap(&mut is_silent, &mut meta.silenced);
        let silent = meta.gen_silent();
        let translation = translate_interpolated_region(self.strings.clone(), interps, false);
        swap(&mut is_silent, &mut meta.silenced);
        let translation = format!("{translation}{silent}");
        if is_statement {
            return if failed.is_empty() { translation } else {
                meta.stmt_queue.push_back(translation);
                failed
            }
        }
        if failed.is_empty() {
            meta.gen_subprocess(&translation)
        } else {
            let id = meta.gen_value_id();
            let quote = meta.gen_quote();
            let dollar = meta.gen_dollar();
            meta.stmt_queue.push_back(format!("__AMBER_VAL_{id}=$({translation})"));
            meta.stmt_queue.push_back(failed);
            format!("{quote}{dollar}{{__AMBER_VAL_{id}}}{quote}")
        }
    }

    pub fn translate_command_statement(&self, meta: &mut TranslateMetadata) -> String {
        self.translate_command(meta, true)
    }
}

impl TranslateModule for Command {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        self.translate_command(meta, false)
    }
}

impl DocumentationModule for Command {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
