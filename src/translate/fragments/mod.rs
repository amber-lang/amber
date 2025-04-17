pub mod block;
pub mod comment;
pub mod eval;
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

// Returns a variable name that should be rendered
pub(self) fn get_variable_name(name: &str, global_id: Option<usize>) -> String {
    match global_id {
        Some(id) => format!("__{id}_{}", name.trim_start_matches("__")),
        None => name.to_string()
    }
}
