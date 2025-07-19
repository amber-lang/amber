use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{docs::module::DocumentationModule, modules::types::{Type, Typed}, utils::metadata::{ParserMetadata, TranslateMetadata}};
use crate::modules::prelude::*;
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Number {
    value: String
}

impl Typed for Number {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Number {
    syntax_name!("Number");

    fn new() -> Self {
        Number {
            value: String::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Ok(sym) = token(meta, "-") {
            self.value.push_str(&sym);
        }
        if let Ok(value) = integer(meta, vec![]) {
            self.value.push_str(&value);
        }
        let sym = token(meta, ".")?;
        self.value.push_str(&sym);
        self.value.push_str(&integer(meta, vec![])?);
        Ok(())
    }
}

impl TranslateModule for Number {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        RawFragment::from(self.value.to_string()).to_frag()
    }
}

impl Number {
    pub fn get_integer_value(&self) -> Option<isize> {
        let value = self.value.parse().unwrap_or_default();
        Some(value)
    }
}

impl DocumentationModule for Number {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
