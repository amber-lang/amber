use heraclitus_compiler::prelude::*;

#[derive(Debug)]
pub struct Bool {
    value: bool
}

impl SyntaxModule<DefaultMetadata> for Bool {
    fn new() -> Self {
        Bool {
            value: false
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        let value = token_by(meta, |value| vec!["true", "false"].contains(&value.as_str()))?;
        self.value = value == "true";
        Ok(())        
    }
}