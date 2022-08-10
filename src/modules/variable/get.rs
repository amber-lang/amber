use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::{Type, Typed}};
use crate::translate::module::TranslateModule;
use super::{variable_name_extensions, handle_variable_reference};

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
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        self.kind = handle_variable_reference(meta, tok, self.name.clone());
        Ok(())
    }
}

impl TranslateModule for VariableGet {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        format!("${}", self.name)
    }
}