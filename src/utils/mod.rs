pub mod cc_flags;
pub mod context;
pub mod function_cache;
pub mod function_interface;
pub mod function_metadata;
pub mod import_cache;
pub mod metadata;

pub use metadata::*;

pub fn pluralize<'a>(amount: usize, single: &'a str, multiple: &'a str) -> &'a str {
    if amount > 1 {
        multiple
    } else {
        single
    }
}