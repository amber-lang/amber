pub mod metadata;
pub mod memory;
pub mod error;
pub mod function_map;
pub mod exports;
pub mod import_history;
pub use metadata::*;

#[macro_export]
macro_rules! context {
    ($body:block, |$name:ident| $error:block) => {
        {
            let ctx: SyntaxResult = (|| { $body })();
            if let Err($name) = ctx {
                $error
            }
        }
    };
}