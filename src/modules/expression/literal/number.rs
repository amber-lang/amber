use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::{ParserMetadata, TranslateMetadata}, modules::{Typed, Type}};
use crate::translate::module::TranslateModule;

#[derive(Debug)]
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
            value: format!("")
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value = number(meta, vec![])?;
        Ok(())
    }
}

impl TranslateModule for Number {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        format!("{}", self.value)
    }
}