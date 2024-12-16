use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{handle_index_accessor, handle_variable_reference, variable_name_extensions};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableGet {
    pub name: String,
    kind: Type,
    global_id: Option<usize>,
    index: Box<Option<Expr>>,
    is_ref: bool
}

impl Typed for VariableGet {
    fn get_type(&self) -> Type {
        match (&self.kind, self.index.as_ref()) {
            (Type::Array(kind), Some(index)) if matches!(index.value, Some(ExprType::Range(_))) => {
                // Array type (indexing array by range)
                Type::Array(kind.clone())
            }
            (Type::Array(kind), Some(_)) => {
                // Item type (indexing array by number)
                *kind.clone()
            }
            (Type::Array(kind), None) => {
                // Array type (returning array)
                Type::Array(kind.clone())
            }
            (kind, _) => {
                // Variable type (returning text or number)
                kind.clone()
            }
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
        let variable = handle_variable_reference(meta, &tok, &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        self.kind = variable.kind.clone();
        self.index = Box::new(handle_index_accessor(meta, true)?);
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
        // Text variables need to be encapsulated in string literals
        // Otherwise, they will be "spread" into tokens
        let quote = meta.gen_quote();
        match &self.kind {
            Type::Array(_) if self.is_ref => {
                if let Some(index) = self.index.as_ref() {
                    let id = meta.gen_value_id();
                    let value = Self::slice_ref_array(meta, &name, index);
                    let stmt = format!("eval \"local __AMBER_ARRAY_GET_{id}_{name}=\\\"\\${{{value}}}\\\"\"");
                    meta.stmt_queue.push_back(stmt);
                    format!("$__AMBER_ARRAY_GET_{id}_{name}")
                } else {
                    format!("{quote}${{!__AMBER_ARRAY_{name}}}{quote}")
                }
            }
            Type::Array(_) if !self.is_ref => {
                if let Some(index) = self.index.as_ref() {
                    let value = Self::slice_copy_array(meta, &name, index);
                    format!("{quote}{value}{quote}")
                } else {
                    format!("{quote}${{{name}[@]}}{quote}")
                }
            }
            Type::Text => {
                let prefix = if self.is_ref { "!" } else { "" };
                format!("{quote}${{{prefix}{name}}}{quote}")
            }
            _ => {
                let prefix = if self.is_ref { "!" } else { "" };
                format!("${{{prefix}{name}}}")
            }
        }
    }
}

impl VariableGet {
    pub fn get_translated_name(&self) -> String {
        match self.global_id {
            Some(id) => format!("__{id}_{}", self.name),
            None => self.name.to_string()
        }
    }

    fn slice_ref_array(meta: &mut TranslateMetadata, name: &str, index: &Expr) -> String {
        match &index.value {
            Some(ExprType::Range(range)) => {
                let (offset, length) = range.get_array_index(meta);
                format!("${name}[@]:{offset}:{length}")
            }
            Some(ExprType::Neg(neg)) => {
                let index = neg.get_array_index(meta);
                format!("${name}[{index}]")
            }
            _ => {
                let index = index.translate_eval(meta, true);
                format!("${name}[{index}]")
            }
        }
    }

    fn slice_copy_array(meta: &mut TranslateMetadata, name: &str, index: &Expr) -> String {
        match &index.value {
            Some(ExprType::Range(range)) => {
                let (offset, length) = range.get_array_index(meta);
                format!("${{{name}[@]:{offset}:{length}}}")
            }
            Some(ExprType::Neg(neg)) => {
                let index = neg.get_array_index(meta);
                format!("${{{name}[{index}]}}")
            }
            _ => {
                let index = index.translate(meta);
                format!("${{{name}[{index}]}}")
            }
        }
    }
}

impl DocumentationModule for VariableGet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
