use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{handle_index_accessor, handle_variable_reference, variable_name_extensions, validate_index_accessor};
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableGet {
    pub name: String,
    kind: Type,
    global_id: Option<usize>,
    index: Box<Option<Expr>>,
    is_ref: bool,
    tok: Option<Token>
}

impl Typed for VariableGet {
    fn get_type(&self) -> Type {
        if let Some(index) = self.index.as_ref() {
            match (&index.value, &self.kind) {
                (Some(ExprType::Range(_)), _) => self.kind.clone(),
                (Some(_), Type::Array(item_type)) => *item_type.clone(),
                _ => self.kind.clone(),
            }
        } else {
            self.kind.clone()
        }
    }
}

impl VariableGet {
    pub fn is_variable_modified(&self) -> bool {
        self.index.is_some()
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
            is_ref: false,
            tok: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        self.index = Box::new(handle_index_accessor(meta, true)?);
        Ok(())
    }
}

impl TypeCheckModule for VariableGet {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let variable = handle_variable_reference(meta, &self.tok, &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        self.kind = variable.kind.clone();
        meta.mark_var_used(&self.name);

        // Typecheck and validate index expression if present
        if let Some(ref mut index_expr) = self.index.as_mut() {
            // Check if the variable can be indexed
            if !matches!(variable.kind, Type::Array(_)) {
                return error!(meta, self.tok.clone(), format!("Cannot index a non-array variable of type '{}'", self.kind));
            }

            // Typecheck the index expression
            index_expr.typecheck(meta)?;

            // Validate the index type
            validate_index_accessor(meta, index_expr, true, self.tok.clone())?;
        }

        Ok(())
    }
}

impl TranslateModule for VariableGet {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new(&self.name, self.get_type())
            .with_global_id(self.global_id)
            .with_ref(self.is_ref)
            .with_index_by_expr(meta, *self.index.clone())
            .to_frag()
    }
}

impl DocumentationModule for VariableGet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
