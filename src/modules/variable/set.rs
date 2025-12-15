use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::{modules::expression::expr::Expr, translate::module::TranslateModule};
use crate::utils::{ParserMetadata, TranslateMetadata};
use super::{handle_index_accessor, handle_variable_reference, prevent_constant_mutation, variable_name_extensions, validate_index_accessor};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct VariableSet {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    index: Option<Expr>,
    is_ref: bool,
    var_type: Type,
    tok: Option<Token>,
}

impl SyntaxModule<ParserMetadata> for VariableSet {
    syntax_name!("Variable Set");

    fn new() -> Self {
        VariableSet {
            name: String::new(),
            expr: Box::new(Expr::new()),
            global_id: None,
            index: None,
            is_ref: false,
            var_type: Type::Null,
            tok: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        self.index = handle_index_accessor(meta, false)?;

        token(meta, "=")?;
        syntax(meta, &mut *self.expr)?;
        Ok(())
    }
}

impl TypeCheckModule for VariableSet {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        if let Some(index) = &mut self.index {
            index.typecheck(meta)?;
        }

        let variable = handle_variable_reference(meta, &self.tok, &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        self.var_type = variable.kind.clone();
        prevent_constant_mutation(meta, &self.tok, &self.name, variable.is_const)?;
        meta.mark_var_modified(&self.name);

        if let Some(ref index_expr) = self.index {
            if !matches!(variable.kind, Type::Array(_)) {
                let left_type = variable.kind.clone();
                return error!(meta, self.tok.clone(), format!("Cannot assign a value to an index of a non-array variable of type '{left_type}'"));
            }

            // Validate the index type (must be integer, not range, for assignment)
            validate_index_accessor(meta, index_expr, false, PositionInfo::from_token(meta, self.tok.clone()))?;
        }

        let expr_type = self.expr.get_type();

        if self.index.is_some() {
            if let Type::Array(kind) = &self.var_type {
                // Handle type inference
                if **kind == Type::Generic {
                    let new_type = Type::array_of(expr_type.clone());
                    meta.update_var_type(&self.name, new_type.clone());
                    self.var_type = new_type;
                    return Ok(());
                }

                if !expr_type.is_allowed_in(kind) {
                    let tok = self.expr.get_position();
                    return error_pos!(meta, tok, format!("Cannot assign value of type '{expr_type}' to an array of '{kind}'"));
                }
            }
        } else {
            // Check for type inference
            if let (Type::Array(inner_var), Type::Array(inner_right)) = (&self.var_type, &expr_type) {
                if **inner_var == Type::Generic && **inner_right != Type::Generic {
                    meta.update_var_type(&self.name, expr_type.clone());
                    self.var_type = expr_type;
                    return Ok(());
                }
            }

            if !expr_type.is_allowed_in(&self.var_type) {
                let tok = self.expr.get_position();
                return error_pos!(meta, tok, format!("Cannot assign value of type '{expr_type}' to a variable of type '{}'", self.var_type));
            }
        }

        Ok(())
    }
}

impl TranslateModule for VariableSet {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let index = self.index.as_ref().map(|v| v.translate(meta));
        let expr = self.expr.translate(meta);
        VarStmtFragment::new(&self.name, self.expr.get_type(), expr)
            .with_global_id(self.global_id)
            .with_ref(self.is_ref)
            .with_index(index)
            .to_frag()
    }
}

impl DocumentationModule for VariableSet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
