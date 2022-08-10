use crate::utils::TranslateMetadata;

pub trait TranslateModule {
    fn translate(&self, meta: &mut TranslateMetadata) -> String;
}