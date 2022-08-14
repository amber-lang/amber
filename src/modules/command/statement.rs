use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::{Type, Typed}};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;

use crate::modules::expression::literal::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug)]
pub struct CommandStatement {
    strings: Vec<String>,
    interps: Vec<Expr>
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
            interps: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) = parse_interpolated_region(meta, '$')?;
        Ok(())
    }
}

impl TranslateModule for CommandStatement {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        format!("{}", translate_interpolated_region(self.strings.clone(), interps.clone()))
    }
}