use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::{ParserMetadata, TranslateMetadata}};
use crate::translate::module::TranslateModule;
use crate::modules::expression::expr::Expr;

use super::{parse_interpolated_region, translate_interpolated_region};

#[derive(Debug, Clone)]
pub struct Text {
    strings: Vec<String>,
    interps: Vec<Expr>,
    escaped: bool,
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
            escaped: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) = parse_interpolated_region(meta, '"')?;
        self.escaped = meta.context.is_escaped_ctx;
        Ok(())
    }
}

impl TranslateModule for Text {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta))
            .collect::<Vec<String>>();
        let strings = translate_interpolated_region(self.strings.clone(), interps, true);
        if self.escaped {
            strings.replace(" ", "\\ ").replace(";", "\\;")
        } else {
            let quote = meta.gen_quote();
            format!("{quote}{strings}{quote}")
        }
    }
}

impl DocumentationModule for Text {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
