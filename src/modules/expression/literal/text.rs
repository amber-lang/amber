use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::{ParserMetadata, TranslateMetadata}};
use crate::modules::prelude::*;
use crate::translate::module::TranslateModule;
use crate::modules::expression::expr::Expr;

use super::parse_interpolated_region;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    strings: Vec<String>,
    interps: Vec<Expr>,
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
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Translate all interpolations
        let interps = self.interps.iter()
            .map(|item| item.translate(meta).with_quotes(false))
            .collect::<Vec<FragmentKind>>();
        InterpolableFragment::new(self.strings.clone(), interps, InterpolableRenderType::StringLiteral).to_frag()
    }
}

impl DocumentationModule for Text {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
