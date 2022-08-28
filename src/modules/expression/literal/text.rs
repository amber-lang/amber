use heraclitus_compiler::prelude::*;
use crate::{utils::{TranslateMetadata, ParserMetadata}, modules::{Type, Typed}};
use crate::translate::module::TranslateModule;
use crate::modules::expression::expr::Expr;

use super::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug)]
pub struct Text {
    strings: Vec<String>,
    interps: Vec<Expr>
}

impl Typed for Text {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Text {
    syntax_name!("Text");

    fn new() -> Self {
        Text {
            strings: vec![],
            interps: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) = parse_interpolated_region(meta, '\'')?;
        Ok(())
    }
}

impl TranslateModule for Text {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        format!("\"{}\"", translate_interpolated_region(self.strings.clone(), interps, true))
    }
}