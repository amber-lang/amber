use crate::modules::prelude::*;
use crate::raw_fragment;
use heraclitus_compiler::prelude::*;
use crate::{fragments, modules::types::Type, utils::ParserMetadata};

pub mod module;
pub mod fragments;
pub mod compute;

pub fn check_all_blocks(meta: &ParserMetadata) -> SyntaxResult {
    let mut stack = 0;
    for token in meta.context.expr.iter() {
        match token.word.as_str() {
            "{" => stack += 1,
            "}" => stack -= 1,
            _ => ()
        }
        if stack < 0 {
            return error!(meta, Some(token.clone()), "Unexpected closing bracket")
        }
    }
    Ok(())
}

/// Create an intermediate variable and return it's statement and
/// expression representing the intermediate variable.
pub fn gen_intermediate_variable(
    name: &str,
    id: Option<usize>,
    kind: Type,
    is_ref: bool,
    index: Option<FragmentKind>,
    op: &str,
    value: FragmentKind
) -> (FragmentKind, VarFragment) {
    let is_array = kind.is_array();
    let variable = VarFragment::new(name, kind, is_ref, id);
    let frags = {
        let mut result = vec![];
        match is_ref {
            true => result.push(raw_fragment!("${{{}}}", variable.get_name())),
            false => result.push(raw_fragment!("{}", variable.get_name())),
        }
        if let Some(index) = index {
            result.push(fragments!("[", index, "]"));
        }
        result.push(raw_fragment!("{}", op));
        if is_array {
            result.push(raw_fragment!("("));
        }
        result.push(value);
        if is_array {
            result.push(raw_fragment!(")"));
        }
        result
    };
    let stmt = ListFragment::new(frags).to_frag();
    (EvalFragment::new(stmt, is_ref).to_frag(), variable)
}

/// Create an intermediate variable only if it makes sense to do so.
pub fn gen_intermediate_variable_lazy(
    name: &str,
    id: Option<usize>,
    kind: Type,
    is_ref: bool,
    index: Option<FragmentKind>,
    op: &str,
    value: FragmentKind
) -> (FragmentKind, VarFragment) {
    match value {
        // If the value is already variable, then we don't need to assign it to a new variable.
        FragmentKind::Var(var) => {
            (FragmentKind::Empty, var)
        },
        _ => {
            gen_intermediate_variable(name, id, kind, is_ref, index, op, value)
        }
    }
}
