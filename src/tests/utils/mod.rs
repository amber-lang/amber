//! Test utilities and helpers for Amber test suite
//!
//! Each test file corresponds to a source file in src/utils/:
//! - cc_flags.rs → src/utils/cc_flags.rs
//! - context.rs → src/utils/context.rs
//! - function_cache.rs → src/utils/function_cache.rs
//! - function_interface.rs → src/utils/function_interface.rs
//! - function_metadata.rs → src/utils/function_metadata.rs
//! - import_cache.rs → src/utils/import_cache.rs
//! - mod_fn_tests.rs → src/utils/mod.rs
//! - ephemeral_vars.rs → src/optimizer/ephemeral_vars.rs
//! - unused_vars.rs → src/optimizer/unused_vars.rs

pub mod fixtures;

// Re-export commonly used helpers for convenience
pub use fixtures::*;

mod cc_flags;
mod context;
mod ephemeral_vars;
mod function_cache;
mod function_interface;
mod function_metadata;
mod import_cache;
mod mod_fn;
mod unused_vars;
