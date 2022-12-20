use heraclitus_compiler::prelude::*;
use crate::{utils::{ParserMetadata, TranslateMetadata}, modules::{types::{Type, Typed}, expression::expr::Expr}};
use crate::translate::module::TranslateModule;
use super::{variable_name_extensions, handle_variable_reference, handle_index_accessor};

#[derive(Debug, Clone)]
pub struct VariableGet {
    pub name: String,
    kind: Type,
    global_id: Option<usize>,
    index: Box<Option<Expr>>,
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
        match *self.index {
            // Return the type of the array element if indexed
            Some(_) => match self.kind.clone() {
                Type::Array(kind) => *kind,
                // This should never happen
                _ => Type::Null
            },
            None => self.kind.clone()
        }
    }
}

impl SyntaxModule<ParserMetadata> for VariableGet {
    syntax_name!("Variable Access");

    fn new() -> Self {
        VariableGet {
            name: String::new(),
            kind: Type::Null,
            global_id: None,
            index: Box::new(None),
            is_ref: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        let variable = handle_variable_reference(meta, tok.clone(), &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        self.kind = variable.kind.clone();
        self.index = Box::new(handle_index_accessor(meta)?);
        // Check if the variable can be indexed
        if self.index.is_some() && !matches!(variable.kind, Type::Array(_)) {
            return error!(meta, tok, format!("Cannot index a non-array variable of type '{}'", self.kind));
        }
        Ok(())
    }
}

impl TranslateModule for VariableGet {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.get_translated_name();
        let ref_prefix = if self.is_ref { "!" } else { "" };
        let res = format!("${{{ref_prefix}{name}}}");
        // Text variables need to be encapsulated in string literals
        // Otherwise, they will be "spreaded" into tokens
        let eval_esc = if meta.eval_ctx { "\\\"" } else { "" };
        match self.kind {
            Type::Text => format!("\"{eval_esc}{res}{eval_esc}\""),
            Type::Array(_) => match *self.index {
                Some(ref expr) => {
                    let index = expr.translate(meta);
                    format!("\"{eval_esc}${{{ref_prefix}{name}[{index}]}}{eval_esc}\"")
                },
                None => format!("\"{eval_esc}${{{ref_prefix}{name}[@]}}{eval_esc}\"")
            },
            _ => res
        }
    }
}