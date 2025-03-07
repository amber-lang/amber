use crate::utils::TranslateMetadata;
use super::fragment::{TranslationFragment, TranslationFragmentable};

/// Represents a comment fragment in the translation process.

#[derive(Debug, Clone)]
pub struct CommentFragment {
    pub value: String,
}

impl CommentFragment {
    pub fn new(value: &str) -> Self {
        CommentFragment {
            value: value.to_string(),
        }
    }
}

impl TranslationFragmentable for CommentFragment {
    fn render(self, _meta: &mut TranslateMetadata) -> String {
        "# ".to_string() + &self.value
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Comment(self)
    }
}
