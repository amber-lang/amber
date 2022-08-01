use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;

#[derive(Debug)]
pub struct Bool {
    value: bool
}

impl SyntaxModule<ParserMetadata> for Bool {
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