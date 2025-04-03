pub mod block;
pub mod fragment;
pub mod raw;
pub mod var;
pub mod interpolable;
pub mod list;
pub mod eval;
pub mod subprocess;
pub mod comment;

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
    (raw: $($args:expr),+) => {
        RawFragment::new(&format!($($args),+)).to_frag()
    };
    (@internal $val:literal) => {
        RawFragment::new($val).to_frag()
    };
    (@internal $val:expr) => {
        $val
    };
}
