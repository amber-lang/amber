/// This is a prelude module that re-exports all the necessary modules for syntax modules
pub use crate::translate::fragments::fragment::TranslationFragmentable;
pub use crate::translate::fragments::{
    block::BlockFragment, compound::CompoundFragment, fragment::TranslationFragment,
    raw::RawFragment, var::VarFragment, var::VarRenderType,
    interpolable::InterpolableFragment, interpolable::InterpolableRenderType,
    list::ListFragment, subprocess::SubprocessFragment
};
pub use crate::translate::module::TranslateModule;
pub use crate::docs::module::DocumentationModule;
pub use crate::utils::{ParserMetadata, TranslateMetadata};
