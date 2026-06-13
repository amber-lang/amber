use crate::utils::ParserMetadata;

/// Implements DocumentationModule with an empty string for types that have no documentation.
#[macro_export]
macro_rules! impl_documentation_noop {
    ($($t:ty),* $(,)?) => {
        $(
            impl $crate::docs::module::DocumentationModule for $t {
                fn document(&self, _meta: &$crate::utils::ParserMetadata) -> String {
                    String::new()
                }
            }
        )*
    };
}

pub trait DocumentationModule {
    fn document(&self, meta: &ParserMetadata) -> String;
}
