pub mod arithmetic;
pub mod block;
pub mod comment;
pub mod condition;
pub mod fragment;
pub mod function_call;
pub mod function_decl;
pub mod interpolable;
pub mod list;
pub mod log;
pub mod raw;
pub mod subprocess;
pub mod var_expr;
pub mod var_stmt;

use crate::{
    modules::function::core::signature::{FunctionDeclId, FunctionVariantId},
    utils::is_all_caps,
};

#[macro_export]
macro_rules! fragments {
    ($($token:expr),+) => {
        ListFragment::new(vec![
            $(fragments!(@internal $token)),*
        ]).to_frag()
    };
    ($token:expr) => {
        fragments!(@internal $token)
    };
    (@internal $val:literal) => {
        RawFragment::new($val).to_frag()
    };
    (@internal $val:expr) => {
        $val
    };
}

#[macro_export]
macro_rules! raw_fragment {
    ($($args:expr),+) => {
        RawFragment::from(format!($($args),+)).to_frag()
    };
}

#[macro_export]
macro_rules! eval_context {
    ($meta:expr, $value:expr, $body:block) => {{
        let temp = $meta.eval_ctx;
        $meta.eval_ctx = $value;
        let result = $body;
        $meta.eval_ctx = temp;
        result
    }};
}

/// Uppercase names are prefixed so they cannot collide with user constants.
fn prefix(name: &str) -> &'static str {
    if is_all_caps(name) {
        "__"
    } else {
        ""
    }
}

// Returns a variable name that should be rendered
pub fn get_variable_name(name: &str, global_id: Option<usize>) -> String {
    match global_id {
        Some(id) => format!("{}{name}_{id}", prefix(name)),
        None => format!("{}{name}", prefix(name)),
    }
}

// Returns a function name that should be rendered
pub fn get_function_name(
    name: &str,
    decl_id: FunctionDeclId,
    variant_id: FunctionVariantId,
) -> String {
    format!("{}{name}__{decl_id}_v{variant_id}", prefix(name))
}

/// The variable to which the function writes its return value into.
pub fn return_variable_name(
    name: &str,
    decl_id: FunctionDeclId,
    variant_id: FunctionVariantId,
) -> String {
    format!("{}ret_{name}__{decl_id}_v{variant_id}", prefix(name))
}

/// Variable which stores the function result on the callsite
/// This is important because of possible multiple calls of the same function in a single expression
pub fn returned_value_variable_name(
    name: &str,
    decl_id: FunctionDeclId,
    variant_id: FunctionVariantId,
    line: usize,
    col: usize,
) -> String {
    format!(
        "{}__{}_{}",
        return_variable_name(name, decl_id, variant_id),
        line,
        col
    )
}

#[cfg(test)]
mod tests {
    use super::get_variable_name;

    #[test]
    fn variable_name_includes_global_id() {
        assert_eq!(get_variable_name("value", Some(7)), "value_7");
        assert_eq!(get_variable_name("VALUE", Some(7)), "__VALUE_7");
    }

    #[test]
    fn uppercase_global_variable_uses_internal_prefix() {
        assert_eq!(get_variable_name("EXEC_SHELL", None), "__EXEC_SHELL");
    }

    #[test]
    fn positional_parameter_is_not_prefixed() {
        assert_eq!(get_variable_name("1", None), "1");
    }

    #[test]
    fn lowercase_name_without_global_id_is_unchanged() {
        assert_eq!(get_variable_name("value", None), "value");
    }
}
