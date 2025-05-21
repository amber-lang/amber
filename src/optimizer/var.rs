use crate::modules::prelude::*;
use crate::translate::fragments::var_expr::VarIndexValue;

type VarExprName = String;
type VarStmtName = String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum UsageType {
    Absolute(VarExprName),
    Relative(VarExprName, VarStmtName),
}

pub struct UnusedVariablesMetadata {
    used_vars: Vec<UsageType>
}

impl UnusedVariablesMetadata {
    pub fn is_used(&self, name: VarStmtName) -> bool {
        let mut transitive_variables = vec![name.clone()];
        for usage_type in self.used_vars.iter() {
            match usage_type {
                UsageType::Absolute(var_expr) => if transitive_variables.contains(var_expr) {
                    return true;
                }
                UsageType::Relative(var_expr, var_stmt) => {
                    if transitive_variables.contains(var_expr) {
                        transitive_variables.push(var_stmt.clone());
                    }
                }
            }
        }
        false
    }
}

impl Default for UnusedVariablesMetadata {
    fn default() -> Self {
        Self {
            used_vars: Vec::new()
        }
    }
}

pub fn remove_unused_variables(ast: &mut FragmentKind) {
    let mut meta = UnusedVariablesMetadata::default();
    find_unused_variables(ast, &mut meta, None);
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
                    return meta.is_used(var_stmt.get_name());
                }
                true
            });
        }
        _ => {}
    }
}

fn find_unused_variables(ast: &FragmentKind, meta: &mut UnusedVariablesMetadata, var_stmt_name: Option<&VarStmtName>) {
    match ast {
        FragmentKind::Block(block) => {
            for statement in block.statements.iter() {
                find_unused_variables(statement, meta, var_stmt_name);
            }
        }
        FragmentKind::List(list) => {
            for item in list.values.iter() {
                find_unused_variables(item, meta, var_stmt_name);
            }
        }
        FragmentKind::Interpolable(interpolable) => {
            for item in interpolable.interps.iter() {
                find_unused_variables(item, meta, var_stmt_name);
            }
        }
        FragmentKind::VarStmt(var_stmt) => {
            find_unused_variables(&var_stmt.value, meta, Some(&var_stmt.get_name()));
        }
        FragmentKind::VarExpr(var_expr) => {
            let variabe_usage = if let Some(stmt_name) = var_stmt_name {
                UsageType::Relative(var_expr.get_name(), stmt_name.to_string())
            } else {
                UsageType::Absolute(var_expr.get_name())
            };
            meta.used_vars.push(variabe_usage);
            if let Some(index) = &var_expr.index {
                match index.as_ref() {
                    VarIndexValue::Index(index) => find_unused_variables(index, meta, var_stmt_name),
                    VarIndexValue::Range(start, end) => {
                        find_unused_variables(start, meta, var_stmt_name);
                        find_unused_variables(end, meta, var_stmt_name);
                    }
                }
            }
        }
        FragmentKind::Subprocess(subprocess) => {
            find_unused_variables(&subprocess.fragment, meta, var_stmt_name);
        }
        _ => {}
    }
}
