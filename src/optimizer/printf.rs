use crate::modules::prelude::*;
use crate::translate::fragments::interpolable::InterpolableRenderType;
use crate::translate::fragments::var_expr::VarIndexValue;

// This optimizer replaces printf with echo when the text is a literal and does not start with a dash (-) character.

const PRINTF_LITERAL_PREFIX: &str = "printf '%s\\n' ";

pub fn optimize_printf_literals(ast: &mut FragmentKind) {
    optimize_fragment(ast);
}

fn optimize_fragment(fragment: &mut FragmentKind) {
    match fragment {
        FragmentKind::Block(block) => {
            for statement in block.statements.iter_mut() {
                optimize_fragment(statement);
            }
        }
        FragmentKind::List(list) => {
            try_optimize_printf_list(list);
            for value in list.values.iter_mut() {
                optimize_fragment(value);
            }
        }
        FragmentKind::Interpolable(interpolable) => {
            for interp in interpolable.interps.iter_mut() {
                optimize_fragment(interp);
            }
        }
        FragmentKind::VarStmt(var_stmt) => {
            if let Some(index) = var_stmt.index.as_mut() {
                optimize_fragment(index.as_mut());
            }
            optimize_fragment(var_stmt.value.as_mut());
        }
        FragmentKind::VarExpr(var_expr) => {
            if let Some(index) = var_expr.index.as_mut() {
                match index.as_mut() {
                    VarIndexValue::Index(value) => optimize_fragment(value),
                    VarIndexValue::Range(start, end) => {
                        optimize_fragment(start);
                        optimize_fragment(end);
                    }
                }
            }
            if let Some(default_value) = var_expr.default_value.as_mut() {
                optimize_fragment(default_value);
            }
        }
        FragmentKind::Subprocess(subprocess) => optimize_fragment(subprocess.fragment.as_mut()),
        FragmentKind::Arithmetic(arith) => {
            if let Some(left) = arith.left.as_mut() {
                optimize_fragment(left);
            }
            if let Some(right) = arith.right.as_mut() {
                optimize_fragment(right);
            }
        }
        FragmentKind::Raw(_) | FragmentKind::Comment(_) | FragmentKind::Empty => {}
    }
}

fn try_optimize_printf_list(list: &mut ListFragment) {
    match list.values.as_mut_slice() {
        [FragmentKind::Raw(prefix), FragmentKind::Interpolable(interpolable)]
            if prefix.value == PRINTF_LITERAL_PREFIX && is_safe_literal(interpolable) =>
        {
            prefix.value = "echo ".to_string();
        }
        _ => {}
    }
}

fn is_safe_literal(interpolable: &InterpolableFragment) -> bool {
    if interpolable.render_type != InterpolableRenderType::StringLiteral {
        return false;
    }

    if !interpolable.interps.is_empty() || interpolable.strings.len() != 1 {
        return false;
    }

    let Some(first_chunk) = interpolable.strings.front() else {
        return true;
    };

    !matches!(first_chunk.chars().next(), Some('-'))
}
