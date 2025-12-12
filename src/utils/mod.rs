pub mod cc_flags;
pub mod context;
pub mod function_cache;
pub mod function_interface;
pub mod function_metadata;
pub mod import_cache;
pub mod metadata;

use itertools::Itertools;
use std::fmt::Display;

pub use metadata::*;

pub fn pluralize<'a>(amount: usize, single: &'a str, multiple: &'a str) -> &'a str {
    if amount > 1 {
        multiple
    } else {
        single
    }
}

pub fn pretty_join<T: Display>(items: &[T], op: &str) -> String {
    let mut all_items = items.iter().map(|item| item.to_string()).collect_vec();
    let last_item = all_items.pop();
    let comma_separated = all_items.iter().join(", ");
    if let Some(last) = last_item {
        if items.len() == 1 {
            last
        } else {
            [comma_separated, last].join(&format!(" {op} "))
        }
    } else {
        comma_separated
    }
}

/// Check if a name consists only of uppercase alphabetic characters (and optionally underscores/numbers)
pub fn is_all_caps(name: &str) -> bool {
    name.chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase())
}