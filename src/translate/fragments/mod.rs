pub mod block;
pub mod comment;
pub mod fragment;
pub mod interpolable;
pub mod list;
pub mod raw;
pub mod subprocess;
pub mod var_expr;
pub mod var_stmt;

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
    ($meta:expr, $value:expr, $body:block) => {
        {
            let temp = $meta.eval_ctx;
            $meta.eval_ctx = $value;
            let result = $body;
            $meta.eval_ctx = temp;
            result
        }
    };
}

// Returns a variable name that should be rendered
pub fn get_variable_name(name: &str, global_id: Option<usize>) -> String {
    match global_id {
        Some(id) => format!("__{id}_{}", name.trim_start_matches("__")),
        None => name.to_string()
    }
}
