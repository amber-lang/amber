use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::types::{Type, Typed}};
use crate::translate::module::TranslateModule;
use super::{variable_name_extensions, handle_variable_reference};

#[derive(Debug, Clone)]
pub struct VariableGet {
    pub name: String,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool
}

impl VariableGet {
    pub fn get_translated_name(&self) -> String {
        match self.global_id {
            Some(id) => format!("__{id}_{}", self.name),
            None => self.name.to_string()
        }
    }
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
            name: String::new(),
            kind: Type::Null,
            global_id: None,
            is_ref: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        let variable = handle_variable_reference(meta, tok, &self.name)?;
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        Ok(())
    }
}

impl TranslateModule for VariableGet {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        let name = self.get_translated_name();
        let ref_prefix = if self.is_ref { "!" } else { "" };
        let res = format!("${{{ref_prefix}{name}}}");
        if let Type::Text = self.get_type() {
            format!("\"{}\"", res)
        } else {
            res
        }
    }
}