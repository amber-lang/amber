use crate::modules::prelude::*;

// This optimizer reduces ephemeral variables to the variables that use them.
// Ephemeral variable is a variable that is created internally by a compiler
// just to hold a value for a single expression. Variable statements are
// marked as ephemeral by the compiler in a case when we create an additional
// variable just to hold a temporary value.
//
// We handle two cases:
// 1. (eph = 5; var = eph) -> (var = 5)
// 2. (eph1 = 5; eph2 = eph1; var = eph2) -> (var = 5)

pub fn remove_ephemeral_variables(ast: &mut FragmentKind) {
    if let FragmentKind::Block(block) = ast {
        let mut i = 0;
        while i < block.statements.len() {
            if i + 1 < block.statements.len() {
                let can_optimize =
                    if let (FragmentKind::VarStmt(first), FragmentKind::VarStmt(second)) =
                        (&block.statements[i], &block.statements[i + 1])
                    {
                        if let FragmentKind::VarExpr(expression) = second.value.as_ref() {
                            let is_regular_variable = !expression.is_length
                                && !expression.is_ref
                                && !expression.is_array_to_string
                                && expression.index.is_none();
                            first.is_ephemeral
                                && first.get_name() == expression.get_name()
                                && is_regular_variable
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                if can_optimize {
                    // Get the value from the first statement
                    let value = if let FragmentKind::VarStmt(first) = &block.statements[i] {
                        first.value.clone()
                    } else {
                        unreachable!("Expected VarStmt");
                    };

                    // Update the second statement to use the value from the first
                    if let FragmentKind::VarStmt(second) = &mut block.statements[i + 1] {
                        second.value = value;
                    }

                    // Remove the first statement
                    block.statements.remove(i);

                    // Don't increment i since we removed a statement and need to check the same position again
                    // This handles transitive chains correctly in O(n) time
                    continue;
                }
            }
            i += 1;
        }

        // Recursively optimize nested blocks
        for item in &mut block.statements {
            remove_ephemeral_variables(item);
        }
    }
}
