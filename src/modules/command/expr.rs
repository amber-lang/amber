use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::{Type, Typed}};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;

use crate::modules::expression::literal::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug)]
pub struct CommandExpr {
    strings: Vec<String>,
    interps: Vec<Expr>
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
            interps: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) = parse_interpolated_region(meta, '$')?;
        Ok(())
    }
}

impl TranslateModule for CommandExpr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        format!("$({})", translate_interpolated_region(self.strings.clone(), interps.clone()))
    }
}