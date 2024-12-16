use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::{ParserMetadata, TranslateMetadata}};
use crate::translate::module::TranslateModule;
use crate::modules::expression::expr::Expr;

use super::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug, Clone)]
pub struct Text {
    strings: Vec<String>,
    interps: Vec<Expr>,
}

impl Text {
    pub fn get_literal_text(&self) -> Option<String> {
        if self.strings.len() == 1 && self.interps.len() == 0 {
            self.strings.first().map(String::clone)
        } else {
            None
        }
    }
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
            interps: vec![],
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) = parse_interpolated_region(meta, '"')?;
        Ok(())
    }
}

impl TranslateModule for Text {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        let quote = meta.gen_quote();
        format!("{quote}{}{quote}", translate_interpolated_region(self.strings.clone(), interps, true))
    }
}

impl DocumentationModule for Text {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
