use amber_meta::ContextManager;
use std::collections::{HashMap, HashSet, VecDeque};

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
enum SymbolType {
    Expression(VarExprName),
    Statement(VarStmtName, Vec<VarExprName>),
    ConditionalBlock(CondBlockBehavior),
}

#[derive(Debug, Default, ContextManager)]
pub struct UnusedVariablesMetadata {
    symbols: VecDeque<SymbolType>,
    dependent_variables: Vec<VarExprName>,
    used_variables: HashSet<VarExprName>,
    #[context]
    pub is_var_rhs_ctx: bool,
}

impl UnusedVariablesMetadata {
    pub fn is_var_used(&mut self, name: VarStmtName) -> bool {
        if self.used_variables.contains(&name) {
            return true;
        }
        let mut transitive_variables = HashMap::from([(name.clone(), vec![0_usize])]);
        let mut cond_block_scope: usize = 0;
        for symbol_type in self.symbols.iter() {
            match symbol_type {
                SymbolType::Expression(var_expr) => {
                    if transitive_variables.contains_key(var_expr) {
                        self.used_variables
                            .extend(transitive_variables.keys().cloned());
                        return true;
                    }
                }
                SymbolType::Statement(var_stmt, dependencies) => {
                    // Case when the same variable is self declared (`a=$a`)
                    if transitive_variables.contains_key(var_stmt)
                        && dependencies.contains(var_stmt)
                    {
                        continue;
                    }
                    // Variable statement is being reassigned with some unknown value
                    if let Some(scopes) = transitive_variables.get_mut(var_stmt) {
                        scopes.retain(|&scope| scope != cond_block_scope);
                    }
                    // If dependencies are used by this variable then this variable is also used
                    if dependencies
                        .iter()
                        .any(|dep| transitive_variables.contains_key(dep))
                    {
                        transitive_variables
                            .entry(var_stmt.clone())
                            .or_insert(Vec::new())
                            .push(cond_block_scope);

                        if self.used_variables.contains(var_stmt) {
                            self.used_variables
                                .extend(transitive_variables.keys().cloned());
                            return true;
                        }
                    }
                    // Remove relations to variables that arent used
                    transitive_variables.retain(|_key, field| !field.is_empty());
                }
                SymbolType::ConditionalBlock(CondBlockBehavior::Begin) => {
                    cond_block_scope += 1;
                }
                SymbolType::ConditionalBlock(CondBlockBehavior::End) => {
                    cond_block_scope = cond_block_scope.saturating_sub(1);
                }
            }
        }
        false
    }

    // Remove all symbols until the first variable statement with the given name
    pub fn move_to_var_stmt_init(&mut self, name: &VarStmtName) {
        let mut found = false;
        self.symbols.retain(|usage_type| {
            if let SymbolType::Statement(var_stmt, ..) = usage_type {
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
        && !var_stmt.value.is_mutating()
}

fn find_unused_variables(ast: &FragmentKind, meta: &mut UnusedVariablesMetadata) {
    match ast {
        FragmentKind::Block(block) => {
            if block.is_conditional {
                meta.symbols
                    .push_back(SymbolType::ConditionalBlock(CondBlockBehavior::Begin));
            }
            for statement in block.statements.iter() {
                find_unused_variables(statement, meta);
            }
            if block.is_conditional {
                meta.symbols
                    .push_back(SymbolType::ConditionalBlock(CondBlockBehavior::End));
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
        FragmentKind::Arithmetic(arith) => {
            if let Some(left) = &*arith.left {
                find_unused_variables(left, meta);
            }
            if let Some(right) = &*arith.right {
                find_unused_variables(right, meta);
            }
        }
        FragmentKind::VarStmt(var_stmt) => {
            if should_optimize_var_stmt(var_stmt) {
                meta.with_is_var_rhs_ctx(true, |meta| -> Result<(), ()> {
                    find_unused_variables(&var_stmt.value, meta);
                    Ok(())
                })
                .unwrap();
                let dependencies = meta.dependent_variables.drain(..).collect();
                meta.symbols
                    .push_back(SymbolType::Statement(var_stmt.get_name(), dependencies));
            } else {
                find_unused_variables(&var_stmt.value, meta);
                if let Some(index) = &var_stmt.index {
                    find_unused_variables(index, meta);
                }
            }
        }
        FragmentKind::VarExpr(var_expr) => {
            if meta.is_var_rhs_ctx {
                meta.dependent_variables.push(var_expr.get_name());
            } else {
                meta.symbols
                    .push_back(SymbolType::Expression(var_expr.get_name()));
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
            if let Some(default_value) = &var_expr.default_value {
                find_unused_variables(default_value, meta);
            }
        }
        FragmentKind::Subprocess(subprocess) => {
            find_unused_variables(&subprocess.fragment, meta);
        }
        FragmentKind::Log(log) => find_unused_variables(&log.value, meta),
        FragmentKind::Raw(_) | FragmentKind::Comment(_) | FragmentKind::Empty => {}
    }
}
