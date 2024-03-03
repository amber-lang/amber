use std::mem::swap;
use heraclitus_compiler::prelude::*;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::condition::failed::Failed;
use crate::modules::types::{Type, Typed};
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::translate::module::TranslateModule;
use crate::modules::expression::literal::{parse_interpolated_region, translate_interpolated_region};

use super::modifier::CommandModifier;

#[derive(Debug, Clone)]
pub struct CommandExpr {
    strings: Vec<String>,
    interps: Vec<Expr>,
    modifier: CommandModifier,
    failed: Failed,
    is_silent_expr: bool
}

impl Typed for CommandExpr {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for CommandExpr {
    syntax_name!("CommandExpr");

    fn new() -> Self {
        CommandExpr {
            strings: vec![],
            interps: vec![],
            modifier: CommandModifier::new().parse_expr(),
            failed: Failed::new(),
            is_silent_expr: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        match syntax(meta, &mut self.modifier) {
            // If the command modifier was parsed successfully, then we swap the result
            Ok(_) => {
                if let Some(ExprType::CommandExpr(command)) = &mut self.modifier.expr.value.clone() {
                    swap(command, self);
                    // Retrieve the silent modifier from the command modifier
                    self.is_silent_expr = command.modifier.is_silent;
                }
                Ok(())
            },
            Err(Failure::Loud(err)) => Err(Failure::Loud(err)),
            Err(Failure::Quiet(_)) => {
                let tok = meta.get_current_token();
                (self.strings, self.interps) = parse_interpolated_region(meta, '$')?;
                match syntax(meta, &mut self.failed) {
                    Ok(_) => Ok(()),
                    Err(Failure::Quiet(_)) => error!(meta, tok => {
                        message: "Every command statement must handle failed execution",
                        comment: "You can use '?' in the end to fail the exit code of the command"
                    }),
                    Err(err) => Err(err)
                }
            }
        }
    }
}

impl TranslateModule for CommandExpr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        let failed = self.failed.translate(meta);
        let silent = if self.is_silent_expr { " 2>/dev/null" } else { "" };
        if failed.is_empty() {
            let translation = translate_interpolated_region(self.strings.clone(), interps, false);
            meta.gen_subprocess(&(translation + silent))
        } else {
            let id = meta.gen_value_id();
            let quote = meta.gen_quote();
            let dollar = meta.gen_dollar();
            let translation = translate_interpolated_region(self.strings.clone(), interps, false);
            meta.stmt_queue.push_back(format!("__AMBER_VAL_{id}=$({translation}{silent})"));
            meta.stmt_queue.push_back(failed);
            format!("{quote}{dollar}{{__AMBER_VAL_{id}}}{quote}")
        }
    }
}

impl DocumentationModule for CommandExpr {
    fn document(&self) -> String {
        "".to_string()
    }
}
