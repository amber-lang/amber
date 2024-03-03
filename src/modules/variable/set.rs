use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::{modules::expression::expr::Expr, translate::module::TranslateModule};
use crate::utils::{ParserMetadata, TranslateMetadata};
use super::{variable_name_extensions, handle_variable_reference, handle_index_accessor};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct VariableSet {
    name: String,
    value: Box<Expr>,
    global_id: Option<usize>,
    index: Option<Expr>,
    is_ref: bool
}

impl SyntaxModule<ParserMetadata> for VariableSet {
    syntax_name!("Variable Set");

    fn new() -> Self {
        VariableSet {
            name: String::new(),
            value: Box::new(Expr::new()),
            global_id: None,
            index: None,
            is_ref: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        self.index = handle_index_accessor(meta)?;
        token(meta, "=")?;
        syntax(meta, &mut *self.value)?;
        let variable = handle_variable_reference(meta, tok.clone(), &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        // Typecheck the variable
        let left_type = variable.kind.clone();
        let right_type = self.value.get_type();
        // Check if the variable can be indexed
        if self.index.is_some() && !matches!(variable.kind, Type::Array(_)) {
            return error!(meta, tok, format!("Cannot assign a value to an index of a non-array variable of type '{left_type}'"));
        }
        // Handle index assignment
        if self.index.is_some() {
            // Check if the assigned value is compatible with the array
            if let Type::Array(kind) = variable.kind.clone() {
                if *kind != self.value.get_type() {
                    let right_type = self.value.get_type();
                    return error!(meta, tok, format!("Cannot assign value of type '{right_type}' to an array of '{kind}'"));
                }
            }
        }
        // Check if the variable is compatible with the assigned value
        else if variable.kind != self.value.get_type() {
            return error!(meta, tok, format!("Cannot assign value of type '{right_type}' to a variable of type '{left_type}'"));
        }
        Ok(())
    }
}

impl TranslateModule for VariableSet {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.name.clone();
        let index = self.index.as_ref()
            .map(|index| format!("[{}]", self.is_ref
                .then(|| index.translate_eval(meta, true))
                .unwrap_or_else(|| index.translate(meta))))
            .unwrap_or_default();
        let mut expr = self.is_ref
            .then(|| self.value.translate_eval(meta, true))
            .unwrap_or_else(|| self.value.translate(meta));
        if let Type::Array(_) = self.value.get_type() {
            expr = format!("({})", expr);
        }
        if self.is_ref {
            format!("eval \"${{{name}}}{index}={expr}\"")
        } else {
            match self.global_id {
                Some(id) => format!("__{id}_{name}{index}={expr}"),
                None => format!("{name}{index}={expr}")
            }
        }
    }
}

impl DocumentationModule for VariableSet {
    fn document(&self) -> String {
        "".to_string()
    }
}
