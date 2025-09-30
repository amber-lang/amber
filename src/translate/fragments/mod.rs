pub mod block;
pub mod comment;
pub mod fragment;
pub mod interpolable;
pub mod list;
pub mod raw;
pub mod subprocess;
pub mod arithmetic;
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

// Helper function to check if a name is all uppercase
fn is_all_uppercase(name: &str) -> bool {
    name.chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase())
}

// Returns a variable name that should be rendered
pub fn get_variable_name(name: &str, global_id: Option<usize>) -> String {
    match global_id {
        Some(id) => {
            let clean_name = name.trim_start_matches("__");
            if is_all_uppercase(clean_name) {
                format!("__{clean_name}_{id}")
            } else {
                format!("{clean_name}_{id}")
            }
        }
        None => name.to_string()
    }
}
