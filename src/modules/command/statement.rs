use heraclitus_compiler::prelude::*;
use crate::{modules::{condition::failed::Failed, rdc::scan_append_externs, types::{Type, Typed}}, utils::{ParserMetadata, TranslateMetadata}};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;

use crate::modules::expression::literal::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug, Clone)]
pub struct CommandStatement {
    strings: Vec<String>,
    interps: Vec<Expr>,
    failed: Failed
}

impl Typed for CommandStatement {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for CommandStatement {
    syntax_name!("CommandStatement");

    fn new() -> Self {
        CommandStatement {
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
                comment: "You can use '?' in the end to propagate the failure"
            }),
            Err(err) => Err(err)
        }
    }
}

impl TranslateModule for CommandStatement {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();

        scan_append_externs(self.strings.clone(), meta);
        
        let failed = self.failed.translate(meta);
        let mut translation = translate_interpolated_region(self.strings.clone(), interps, false);
        let silent = meta.gen_silent();
        // Strip down all the inner command interpolations [A32]
        while translation.starts_with("$(") {
            let end = translation.len() - 1;
            translation = translation.get(2..end).unwrap().to_string();
        }
        format!("{translation}{silent}\n{failed}").trim_end().to_string()
    }
}