use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
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
        match (&*self.index, self.kind.clone()) {
            // Return the type of the array element if indexed
            (Some(_), Type::Array(kind)) => *kind,
            _ => self.kind.clone()
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
        // Text variables need to be encapsulated in string literals
        // Otherwise, they will be "spread" into tokens
        let quote = meta.gen_quote();
        match &self.kind {
            Type::Array(_) => {
                if self.is_ref {
                    if let Some(index) = self.index.as_ref() {
                        let value = {
                            let index = index.translate_eval(meta, true);
                            format!("\\\"\\${{${name}[{index}]}}\\\"")
                        };
                        let id = meta.gen_value_id();
                        let stmt = format!("eval \"local __AMBER_ARRAY_GET_{id}_{name}={value}\"");
                        meta.stmt_queue.push_back(stmt);
                        format!("$__AMBER_ARRAY_GET_{id}_{name}") // echo $__ARRAY_GET
                    } else {
                        format!("{quote}${{!__AMBER_ARRAY_{name}}}{quote}")
                    }
                } else {
                    if let Some(index) = self.index.as_ref() {
                        let index = index.translate(meta);
                        format!("{quote}${{{name}[{index}]}}{quote}")
                    } else {
                        format!("{quote}${{{name}[@]}}{quote}")
                    }
                }
            }
            Type::Text => {
                let ref_prefix = if self.is_ref { "!" } else { "" };
                format!("{quote}${{{ref_prefix}{name}}}{quote}")
            }
            _ => {
                let ref_prefix = if self.is_ref { "!" } else { "" };
                format!("${{{ref_prefix}{name}}}")
            }
        }
    }
}

impl DocumentationModule for VariableGet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
