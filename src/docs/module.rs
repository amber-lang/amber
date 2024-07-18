use crate::utils::ParserMetadata;

pub trait DocumentationModule {
    fn document(&self, meta: &ParserMetadata) -> String;
}
