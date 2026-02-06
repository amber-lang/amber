use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

/// Trait for modules that need to perform type checking operations.
/// This separates type checking logic from the parsing logic in SyntaxModule.
pub trait TypeCheckModule {
    /// Perform type checking for this module.
    /// This method should be called after parsing is complete but before translation.
    /// It can mutate the module to store type information.
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}
