use heraclitus_compiler::prelude::*;

#[derive(Debug)]
pub struct Text {
    value: String
}

impl SyntaxModule<DefaultMetadata> for Text {
    fn new() -> Self {
        Text {
            value: format!("")
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        self.value = token_by(meta, |word| word.starts_with('\'') && word.ends_with('\''))?;
        self.value = self.value.chars().take(self.value.len() - 2).skip(1).collect::<String>();
        Ok(())
    }
}