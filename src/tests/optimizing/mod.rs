pub mod unused_vars;

#[macro_export]
macro_rules! bash_code {
    // Base case
    (@acc [$($elems:expr),*]) => {
        vec![$($elems),*]
    };
    // Variable assignments
    (@acc [$($elems:expr),*] $a:ident = $b:ident; $($rest:tt)*) => {{
        let value = VarExprFragment::new(stringify!($b), Type::Generic).to_frag();
        let variable = VarStmtFragment::new(stringify!($a), Type::Generic, value).to_frag();
        bash_code!(@acc [$($elems,)* variable] $($rest)*)
    }};
    (@acc [$($elems:expr),*] $a:ident = $b:literal; $($rest:tt)*) => {{
        let variable = VarStmtFragment::new(stringify!($a), Type::Generic, raw_fragment!(stringify!($b))).to_frag();
        bash_code!(@acc [$($elems,)* variable] $($rest)*)
    }};
    // Blocks
    (@acc [$($elems:expr),*] if { $($cond_block:tt)* } $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* BlockFragment::new(bash_code!({ $($cond_block)* }), true).with_condition(true).to_frag()] $($rest)*)
    };
    (@acc [$($elems:expr),*] { $($cond_block:tt)* } $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* BlockFragment::new(bash_code!({ $($cond_block)* }), true).to_frag()] $($rest)*)
    };
    // Other syntax
    (@acc [$($elems:expr),*] syntax($expr:expr); $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* $expr] $($rest)*)
    };
    // Variable expression
    (@acc [$($elems:expr),*] $var:ident; $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* VarExprFragment::new(stringify!($var), Type::Generic).to_frag()] $($rest)*)
    };
    ({ $($tokens:tt)* }) => {
        bash_code!(@acc [] $($tokens)*)
    };
}
