use super::{handle_variable_reference, prevent_constant_mutation, variable_name_extensions};
use crate::docs::module::DocumentationModule;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::get_default_value_fragment;
use crate::raw_fragment;
use crate::translate::fragments::var_expr::VarIndexValue;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::{modules::expression::expr::Expr, translate::module::TranslateModule};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableSetDestruct {
    names: Vec<String>,
    expr: Box<Expr>,
    global_ids: Vec<Option<usize>>,
    is_refs: Vec<bool>,
    var_types: Vec<Type>,
    toks: Vec<Option<Token>>,
}

impl SyntaxModule<ParserMetadata> for VariableSetDestruct {
    syntax_name!("Variable Set Destruct");

    fn new() -> Self {
        VariableSetDestruct {
            names: Vec::new(),
            expr: Box::new(Expr::new()),
            global_ids: Vec::new(),
            is_refs: Vec::new(),
            var_types: Vec::new(),
            toks: Vec::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "[")?;
        loop {
            let tok = meta.get_current_token();
            let name = variable(meta, variable_name_extensions())?;
            self.names.push(name);
            self.toks.push(tok);

            if token(meta, ",").is_err() {
                break;
            }
        }
        token(meta, "]")?;
        token(meta, "=")?;
        syntax(meta, &mut *self.expr)?;
        Ok(())
    }
}

impl TypeCheckModule for VariableSetDestruct {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;

        // Ensure the expression is an array of known type
        let inner_expr_type = match self.expr.get_type() {
            Type::Array(inner) if *inner == Type::Generic => {
                let pos = self.expr.get_position();
                return error_pos!(meta, pos => {
                    message: "Cannot destructure array because its concrete type is unknown",
                    comment: "Please add an explicit type annotation to this array value before destructuring"
                });
            }
            Type::Array(inner) => *inner,
            _ => {
                let pos = self.expr.get_position();
                return error_pos!(
                    meta,
                    pos,
                    format!(
                        "Destructuring assignment requires an array type, but received '{}'",
                        self.expr.get_type()
                    )
                );
            }
        };

        for (name, tok) in self.names.iter().zip(self.toks.iter()) {
            let variable = handle_variable_reference(meta, tok, name)?;
            self.global_ids.push(variable.global_id);
            self.is_refs.push(variable.is_ref);
            self.var_types.push(variable.kind.clone());

            prevent_constant_mutation(meta, tok, name, variable.is_const)?;
            meta.mark_var_modified(name);

            if let Type::Array(kind) = &variable.kind {
                // Handle type inference for generic arrays or incompatible types
                if **kind == Type::Generic {
                    let new_type = Type::array_of(inner_expr_type.clone());
                    meta.update_var_type(name, new_type);
                    // We need to update our stored type as well to reflect the change
                    if let Some(last) = self.var_types.last_mut() {
                        *last = Type::array_of(inner_expr_type.clone());
                    }
                } else if !inner_expr_type.is_allowed_in(kind) {
                    return error!(
            meta,
            tok.clone(),
            format!("Cannot assign value of type '{inner_expr_type}' to an array of '{kind}'")
          );
                }
            } else if !inner_expr_type.is_allowed_in(&variable.kind) {
                return error!(
                    meta,
                    tok.clone(),
                    format!(
            "Cannot assign value of type '{inner_expr_type}' to a variable of type '{}'",
            variable.kind
          )
                );
            }
        }

        Ok(())
    }
}

impl TranslateModule for VariableSetDestruct {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        let mut fragments = vec![];

        // Assign expression to temp array
        let temp_array_name = format!("array_destruct_{}", meta.gen_value_id());
        let assign_temp = VarStmtFragment::new(&temp_array_name, self.expr.get_type(), expr)
            .with_local(false)
            .with_optimization_when_unused(false);
        fragments.push(assign_temp.clone().to_frag());

        let inner_type = match self.expr.get_type() {
            Type::Array(t) => *t,
            _ => unreachable!("Type of expression is not an array in set destructuring"),
        };

        for (i, name) in self.names.iter().enumerate() {
            let assign_expr = VarExprFragment::from_stmt(&assign_temp)
                .with_index_by_value(VarIndexValue::Index(raw_fragment!("{i}")))
                .with_default_value(get_default_value_fragment(&inner_type))
                .to_frag();

            let assign_var = VarStmtFragment::new(name, inner_type.clone(), assign_expr)
                .with_global_id(self.global_ids[i])
                .with_ref(self.is_refs[i])
                .to_frag();

            fragments.push(assign_var);
        }

        BlockFragment::new(fragments, false).to_frag()
    }
}

impl DocumentationModule for VariableSetDestruct {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
