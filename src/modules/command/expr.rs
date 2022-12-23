use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::{types::{Type, Typed}, condition::failed::Failed}};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;

use crate::modules::expression::literal::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug, Clone)]
pub struct CommandExpr {
    strings: Vec<String>,
    interps: Vec<Expr>,
    failed: Failed
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
            failed: Failed::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
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

impl TranslateModule for CommandExpr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        let failed = self.failed.translate(meta);
        if failed.is_empty() {
            format!("$({})", translate_interpolated_region(self.strings.clone(), interps, false))
        } else {
            let id = meta.gen_value_id();
            let quote = meta.gen_quote();
            let translation = translate_interpolated_region(self.strings.clone(), interps, false);
            meta.stmt_queue.push_back(format!("__AMBER_VAL_{id}=$({translation})"));
            meta.stmt_queue.push_back(failed);
            format!("{quote}${{__AMBER_VAL_{id}}}{quote}")
        }
    }
}