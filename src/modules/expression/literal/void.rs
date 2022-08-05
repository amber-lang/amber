use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};

#[derive(Debug)]
pub struct Void {}

impl Typed for Void {
    fn get_type(&self) -> Type {
        Type::Void
    }
}

impl SyntaxModule<ParserMetadata> for Void {
    syntax_name!("Void");

    fn new() -> Self {
        Void {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "void")?;
        Ok(())        
    }
}