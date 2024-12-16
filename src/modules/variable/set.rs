use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::{modules::expression::expr::Expr, translate::module::TranslateModule};
use crate::utils::{ParserMetadata, TranslateMetadata};
use super::{handle_index_accessor, handle_variable_reference, prevent_constant_mutation, variable_name_extensions};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct VariableSet {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    index: Option<Expr>,
    is_ref: bool
}

impl VariableSet {
    fn translate_eval_if_ref(&self, expr: &Expr, meta: &mut TranslateMetadata) -> String {
        if self.is_ref {
            expr.translate_eval(meta, true)
        } else {
            expr.translate(meta)
        }
    }
}

impl SyntaxModule<ParserMetadata> for VariableSet {
    syntax_name!("Variable Set");

    fn new() -> Self {
        VariableSet {
            name: String::new(),
            expr: Box::new(Expr::new()),
            global_id: None,
            index: None,
            is_ref: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        self.index = handle_index_accessor(meta, false)?;
        token(meta, "=")?;
        syntax(meta, &mut *self.expr)?;
        let variable = handle_variable_reference(meta, &tok, &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        prevent_constant_mutation(meta, &tok, &self.name, variable.is_const)?;
        // Typecheck the variable
        let left_type = variable.kind.clone();
        let right_type = self.expr.get_type();
        // Check if the variable can be indexed
        if self.index.is_some() && !matches!(variable.kind, Type::Array(_)) {
            return error!(meta, tok, format!("Cannot assign a value to an index of a non-array variable of type '{left_type}'"));
        }
        // Handle index assignment
        if self.index.is_some() {
            // Check if the assigned value is compatible with the array
            if let Type::Array(kind) = variable.kind.clone() {
                if *kind != self.expr.get_type() {
                    let right_type = self.expr.get_type();
                    return error!(meta, tok, format!("Cannot assign value of type '{right_type}' to an array of '{kind}'"));
                }
            }
        }
        // Check if the variable is compatible with the assigned value
        else if variable.kind != self.expr.get_type() {
            return error!(meta, tok, format!("Cannot assign value of type '{right_type}' to a variable of type '{left_type}'"));
        }
        Ok(())
    }
}

impl TranslateModule for VariableSet {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.name.clone();
        let index = self.index.as_ref()
            .map(|index| self.translate_eval_if_ref(index, meta))
            .map(|index| format!("[{index}]"))
            .unwrap_or_default();
        let mut expr = self.translate_eval_if_ref(self.expr.as_ref(), meta);
        if let Type::Array(_) = self.expr.get_type() {
            expr = format!("({expr})");
        }
        if let Some(id) = self.global_id {
            format!("__{id}_{name}{index}={expr}")
        } else if self.is_ref {
            format!("eval \"${{{name}}}{index}={expr}\"")
        } else {
            format!("{name}{index}={expr}")
        }
    }
}

impl DocumentationModule for VariableSet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
