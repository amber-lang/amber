use crate::modules::prelude::*;
use crate::translate::fragments::var_expr::VarIndexValue;
use std::collections::HashSet;

pub struct UnusedVariablesMetadata {
    used_vars: HashSet<String>
}

impl Default for UnusedVariablesMetadata {
    fn default() -> Self {
        Self {
            used_vars: HashSet::new()
        }
    }
}

pub fn remove_unused_variables(ast: &mut FragmentKind) {
    let mut meta = UnusedVariablesMetadata::default();
    find_unused_variables(ast, &mut meta);
    remove_non_existing_variables(ast, &mut meta);
}

fn remove_non_existing_variables(ast: &mut FragmentKind, meta: &mut UnusedVariablesMetadata) {
    match ast {
        FragmentKind::Block(block) => {
            for statement in block.statements.iter_mut() {
                remove_non_existing_variables(statement, meta);
            }
            block.statements.retain(|statement| {
                if let FragmentKind::VarStmt(var_stmt) = statement {
                    let result = meta.used_vars.contains(&var_stmt.get_name());
                    return result
                }
                true
            });
        }
        _ => {}
    }
}

fn find_unused_variables(ast: &FragmentKind, meta: &mut UnusedVariablesMetadata) {
    match ast {
        FragmentKind::Block(block) => {
            for statement in block.statements.iter() {
                find_unused_variables(statement, meta);
            }
        }
        FragmentKind::List(list) => {
            for item in list.values.iter() {
                find_unused_variables(item, meta);
            }
        }
        FragmentKind::Interpolable(interpolable) => {
            for item in interpolable.interps.iter() {
                find_unused_variables(item, meta);
            }
        }
        FragmentKind::VarStmt(var_stmt) => {
            find_unused_variables(&var_stmt.value, meta);
        }
        FragmentKind::VarExpr(var_expr) => {
            meta.used_vars.insert(var_expr.get_name());
            if let Some(index) = &var_expr.index {
                match index.as_ref() {
                    VarIndexValue::Index(index) => find_unused_variables(index, meta),
                    VarIndexValue::Range(start, end) => {
                        find_unused_variables(start, meta);
                        find_unused_variables(end, meta);
                    }
                }
            }
        }
        FragmentKind::Subprocess(subprocess) => {
            find_unused_variables(&subprocess.fragment, meta);
        }
        _ => {}
    }
}
