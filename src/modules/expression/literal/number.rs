use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;

#[derive(Debug)]
pub struct Number {
    value: String
}

impl SyntaxModule<ParserMetadata> for Number {
    fn new() -> Self {
        Number {
            value: format!("")
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value = number(meta, vec![])?;
        Ok(())
    }
}