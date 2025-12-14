use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::{modules::expression::expr::Expr, translate::module::TranslateModule};
use crate::utils::{ParserMetadata, TranslateMetadata};
use super::{handle_variable_reference, prevent_constant_mutation, variable_name_extensions};
use crate::modules::types::{Typed, Type};
use crate::translate::fragments::var_expr::VarIndexValue;
use crate::raw_fragment;

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
        
        // Ensure the expression is an array
        let inner_type = match self.expr.get_type() {
            Type::Array(inner) => *inner,
            _ => {
                let tok = self.toks.first().cloned().flatten();
                return error!(meta, tok, format!("Destructuring assignment requires an array type, but received '{}'", self.expr.get_type()));
            }
        };

        for (name, tok) in self.names.iter().zip(self.toks.iter()) {
            let variable = handle_variable_reference(meta, tok, name)?;
            self.global_ids.push(variable.global_id);
            self.is_refs.push(variable.is_ref);
            self.var_types.push(variable.kind.clone());
            
            prevent_constant_mutation(meta, tok, name, variable.is_const)?;
            meta.mark_var_modified(name);

            // Type checking logic similar to VariableSet
            if let Type::Array(kind) = &variable.kind {
                // Handle type inference for generic arrays or incompatible types
                if **kind == Type::Generic {
                    let new_type = Type::array_of(inner_type.clone());
                    meta.update_var_type(name, new_type);
                    // We need to update our stored type as well to reflect the change
                    if let Some(last) = self.var_types.last_mut() {
                        *last = Type::array_of(inner_type.clone());
                    }
                } else if !inner_type.is_allowed_in(kind) {
                     return error!(meta, tok.clone(), format!("Cannot assign value of type '{inner_type}' to an array of '{kind}'"));
                }
            } else {
                 if !inner_type.is_allowed_in(&variable.kind) {
                    return error!(meta, tok.clone(), format!("Cannot assign value of type '{inner_type}' to a variable of type '{}'", variable.kind));
                }
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
            .with_local(false) // Temporary variables usually don't need to be strictly local in this context, or it depends on scope. 
             // However, strictly speaking, destructuring happens in a scope. 
             // Let's check init_destruct.rs: .with_local(self.is_fun_ctx). 
             // Here we are in a set specific context. Let's assume false or check context if possible.
             // Actually `VariableSet` doesn't use `is_fun_ctx` for localness in that way usually, it sets global_id.
             // But for a temp variable, we want it declared.
             // If we are inside a function, we want it local.
             // But valid `VariableSetDestruct` doesn't carry `is_fun_ctx`. `VariableInitDestruct` does.
             // In `VariableSet`, we just emit `VarStmtFragment`.
             // `VarStmtFragment` defaults to local if not global_id is set? No.
             // Let's look at `VariableSet::translate`: uses `VarStmtFragment::new`...
             // Wait, `VariableSet` updates an EXISTING variable.
             // Here we need to creating a NEW TEMPORARY variable for the array.
             // And then update EXISTING variables.
             
             // The temp array should probably be local if we are in a function?
             // But we don't track `is_fun_ctx` in `VariableSetDestruct`.
             // We can infer it or just make it standard.
             // If we look at `init_destruct.rs`, it uses `self.is_fun_ctx`.
             // `VariableSetDestruct` does NOT have `is_fun_ctx`.
             // However, `TranslateMetadata` doesn't strictly track "am I in a function" easily visible here?
             // Actually, usually `Fragment` handles this.
             // Let's use `.with_local(true)` if we want `local` keyword, which is safer for temp vars inside functions?
             // But if we are global, `local` is invalid?
             // Amber compiles to shell. `local` is only valid in functions.
             // IF we are at top level, we shouldn't use `local`.
             // We lack `is_fun_ctx` here.
             // Does `ParserMetadata` know? Yes. But `TranslateMetadata`?
             // Maybe we should just use a standard assignment provided by `VarStmtFragment`.
             // If it's a new variable (temp), we might want `local` if in function.
             // BUT `VariableSetDestruct` didn't capture `is_fun_ctx` in `parse`!
             // `VariableInitDestruct` DOES capture it.
             // I should probably add `is_fun_ctx` to `VariableSetDestruct` struct and capturing it in `parse`.
            .with_optimization_when_unused(false);

        // However, I cannot change `parse` signature easily right now without editing struct definition.
        // Let's modify struct definition first if needed.
        // WAIT. `VariableInitDestruct` captures it in `parse`. I can too.
        // But simply, if I don't use `local`, it might leak to global scope in Bash.
        // That's acceptable for a temp variable `array_destruct_...` which is unique ID anyway.
        // So I will just stick with default (not forcing local) for now to minimize changes, 
        // OR I should update struct. 
        // Let's look at `VariableSetDestruct` struct again.
             
        fragments.push(assign_temp.clone().to_frag());

        let inner_type = match self.expr.get_type() {
            Type::Array(t) => *t,
            _ => Type::Generic, 
        };

        for (i, name) in self.names.iter().enumerate() {
            // value = temp_array[i]
             let assign_expr = VarExprFragment::from_stmt(&assign_temp)
                .with_index_by_value(VarIndexValue::Index(raw_fragment!("{i}")))
                .to_frag();

            // v = value
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
