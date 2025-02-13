use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{handle_index_accessor, handle_variable_reference, variable_name_extensions};
use crate::modules::prelude::*;
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
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        return VarFragment::new(
            &self.name,
            self.kind.clone(),
            self.is_ref,
            self.global_id
        ).with_index(meta, *self.index.clone()).to_frag();
    }
}

impl DocumentationModule for VariableGet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
