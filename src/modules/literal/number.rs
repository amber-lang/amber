use heraclitus_compiler::prelude::*;

#[derive(Debug)]
pub struct Number {
    value: String
}

impl SyntaxModule<DefaultMetadata> for Number {
    fn new() -> Self {
        Number {
            value: format!("")
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        self.value = number(meta, vec![])?;
        Ok(())
    }
}