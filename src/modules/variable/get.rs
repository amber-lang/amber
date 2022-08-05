use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};
use super::variable_name_extensions;

#[derive(Debug)]
pub struct VariableGet {
    name: String,
    kind: Type
}

impl Typed for VariableGet {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for VariableGet {
    syntax_name!("Variable Access");

    fn new() -> Self {
        VariableGet {
            name: format!(""),
            kind: Type::Void
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.name = variable(meta, variable_name_extensions())?;
        // TODO: Get the actual type of the variable
        Ok(())
    }
}