use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};

#[derive(Debug)]
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
        let value = token_by(meta, |value| vec!["true", "false"].contains(&value.as_str()))?;
        self.value = value == "true";
        Ok(())        
    }
}