use crate::modules::prelude::*;

#[derive(Debug, Clone)]
enum VariableAction {
    Remove,
    Reassign(Box<FragmentKind>),
}

// This optimizer reduces ephermal variables to the variables that use them.
// Ephermal variable is a variable that is created internally by a compiler
// just to hold a value for a single expression. Variable statements are
// marked as ephermal by the compiler in a case when we create an additional
// variable just to hold a temporary value.
//
// We handle two cases:
// 1. (eph = 5; var = eph) -> (var = 5)
// 2. (eph1 = 5; eph2 = eph1; var = eph2) -> (var = 5)

pub fn remove_ephermal_variables(ast: &mut FragmentKind) {
    match ast {
        FragmentKind::Block(block) => {
            let mut state = vec![None; block.statements.len()];

            let mut i = 0;
            for window in block.statements.windows(2) {
                if let (FragmentKind::VarStmt(first), FragmentKind::VarStmt(second)) = (&window[0], &window[1]) {
                    if let FragmentKind::VarExpr(expression) = second.value.as_ref() {
                        if first.is_ephermal && first.get_name() == expression.get_name() {
                            match state[i].take() {
                                Some(VariableAction::Reassign(expr)) => {
                                    state[i] = Some(VariableAction::Remove);
                                    state[i + 1] = Some(VariableAction::Reassign(expr.clone()));
                                },
                                _ => {
                                    state[i] = Some(VariableAction::Remove);
                                    state[i + 1] = Some(VariableAction::Reassign(first.value.clone()));
                                }
                            }
                            continue;
                        }
                    }
                }
                i += 1;
            }

            let mut i = 0;
            // Reassign the variables
            for stmt in block.statements.iter_mut() {
                if let FragmentKind::VarStmt(var_stmt) = stmt {
                    match state[i].take() {
                        Some(VariableAction::Reassign(expr)) => {
                            var_stmt.value = expr;
                        }
                        other => {
                            state[i] = other;
                        }
                    }
                }
                i += 1;
            }

            // Remove the variables
            let i = 0;
            block.statements.retain_mut(|_| !matches!(state[i].take(), Some(VariableAction::Remove)));
            for item in &mut block.statements {
                remove_ephermal_variables(item);
            }
        },
        _ => {}
    }
}
