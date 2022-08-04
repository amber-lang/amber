use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;

#[derive(Debug)]
pub struct Text {
    value: String
}

impl SyntaxModule<ParserMetadata> for Text {
    syntax_name!("Text");

    fn new() -> Self {
        Text {
            value: format!("")
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value = token_by(meta, |word| word.starts_with('\'') && word.ends_with('\''))?;
        self.value = self.value.chars().take(self.value.len() - 2).skip(1).collect::<String>();
        Ok(())
    }
}