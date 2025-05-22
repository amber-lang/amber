use std::collections::{HashSet, VecDeque};
use amber_meta::ContextManager;

use crate::modules::prelude::*;
use crate::translate::fragments::var_expr::VarIndexValue;

// This optimizer removes unused variables from the AST in cases of:
// 1. Transitive variables not being used (eg. `a = b; b = c;`)
// 2. Variables being redeclared in certain scopes (non-conditional blocks)

type VarExprName = String;
type VarStmtName = String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum CondBlockBehavior {
    Begin,
    End,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum UsageType {
    Expression(VarExprName),
    Statement(VarStmtName, Vec<VarExprName>),
    ConditionalBlock(CondBlockBehavior),
}

#[derive(Debug, Default, ContextManager)]
pub struct UnusedVariablesMetadata {
    used_vars: VecDeque<UsageType>,
    dependent_variables: Vec<VarExprName>,
    #[context]
    pub is_var_rhs_ctx: bool,
}

impl UnusedVariablesMetadata {
    pub fn is_var_used(&self, name: VarStmtName) -> bool {
        let mut transitive_variables = HashSet::from([name.clone()]);
        let mut cond_blocks = 0;
        for usage_type in self.used_vars.iter() {
            match usage_type {
                UsageType::Expression(var_expr) => if transitive_variables.contains(var_expr) {
                    return true;
                }
                UsageType::Statement(var_stmt, dependencies) => {
                    let dependencies_in_transitive = transitive_variables.iter().any(|t_var| dependencies.contains(t_var));
                    if transitive_variables.contains(var_stmt) && dependencies.contains(var_stmt) {
                        continue;
                    }
                    // If this statement is later used in a conditional block
                    if cond_blocks > 0 {
                        // Don't track variables in conditional blocks
                        if dependencies_in_transitive {
                            return true;
                        }
                        continue;
                    }
                    // Variable statement is being overwritten
                    if let Some(value) = transitive_variables.iter().find(|var| *var == var_stmt).cloned() {
                        transitive_variables.remove(&value);
                    }
                    // If dependencies are used by this variable then this variable is also used
                    if dependencies_in_transitive {
                        transitive_variables.insert(var_stmt.clone());
                    }
                },
                UsageType::ConditionalBlock(CondBlockBehavior::Begin) => {
                    cond_blocks += 1;
                },
                UsageType::ConditionalBlock(CondBlockBehavior::End) => {
                    if cond_blocks > 0 {
                        cond_blocks -= 1;
                    }
                }
            }
        }
        false
    }

    pub fn move_to_var_stmt_init(&mut self, name: &VarStmtName) {
        let mut found = false;
        self.used_vars.retain(|usage_type| {
            if let UsageType::Statement(var_stmt, ..) = usage_type {
                if !found && var_stmt == name {
                    found = true;
                    return false;
                }
            }
            found
        });
    }
}

pub fn remove_unused_variables(ast: &mut FragmentKind) {
    let mut meta = UnusedVariablesMetadata::default();
    find_unused_variables(ast, &mut meta);
    remove_non_existing_variables(ast, &mut meta);
}

fn remove_non_existing_variables(ast: &mut FragmentKind, meta: &mut UnusedVariablesMetadata) {
    if let FragmentKind::Block(block) = ast {
        let mut remove_indexes = vec![];
        for (index, statement) in block.statements.iter_mut().enumerate() {
            if let FragmentKind::VarStmt(var_stmt) = statement {
                if !should_optimize_var_stmt(var_stmt) {
                    continue;
                }
                meta.move_to_var_stmt_init(&var_stmt.get_name());
                if !meta.is_var_used(var_stmt.get_name()) {
                    remove_indexes.push(index);
                }
            } else {
                remove_non_existing_variables(statement, meta);
            }
        }
        // Remove variables that are not used
        for index in remove_indexes.iter().rev() {
            block.statements.remove(*index);
        }
    }
}

fn should_optimize_var_stmt(var_stmt: &VarStmtFragment) -> bool {
    // Refs cannot be optimized because they mutate external environment that could be used later on
    !var_stmt.is_ref
        && var_stmt.optimize_unused
        && var_stmt.index.is_none()
        && var_stmt.operator == "="
}

fn find_unused_variables(ast: &FragmentKind, meta: &mut UnusedVariablesMetadata) {
    match ast {
        FragmentKind::Block(block) => {
            if block.is_conditional {
                meta.used_vars.push_back(UsageType::ConditionalBlock(CondBlockBehavior::Begin));
            }
            for statement in block.statements.iter() {
                find_unused_variables(statement, meta);
            }
            if block.is_conditional {
                meta.used_vars.push_back(UsageType::ConditionalBlock(CondBlockBehavior::End));
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
            if should_optimize_var_stmt(var_stmt) {
                meta.with_is_var_rhs_ctx(true, |meta| -> Result<(), ()> {
                    find_unused_variables(&var_stmt.value, meta);
                    Ok(())
                }).unwrap();
                let dependencies = meta.dependent_variables.drain(..).collect();
                meta.used_vars.push_back(UsageType::Statement(var_stmt.get_name(), dependencies));
            } else {
                find_unused_variables(&var_stmt.value, meta);
            }

        }
        FragmentKind::VarExpr(var_expr) => {
            if meta.is_var_rhs_ctx {
                meta.dependent_variables.push(var_expr.get_name());
            } else {
                meta.used_vars.push_back(UsageType::Expression(var_expr.get_name()));
            }
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
