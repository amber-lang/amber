use heraclitus_compiler::prelude::*;
use crate::raw_fragment;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};

#[derive(Debug, Clone)]
pub struct Bool {
    value: bool
}

impl Typed for Bool {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Bool {
    syntax_name!("Bool");

    fn new() -> Self {
        Bool {
            value: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |value| ["true", "false"].contains(&value.as_str()))?;
        self.value = value == "true";
        Ok(())
    }
}

impl TranslateModule for Bool {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        raw_fragment!("{}", if self.value { 1 } else { 0 })
    }
}

impl DocumentationModule for Bool {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
