use crate::utils::TranslateMetadata;

use super::fragment::{TranslationFragment, TranslationFragmentable};

#[derive(Debug, Clone)]
pub struct RawFragment {
    pub value: String,
}

impl RawFragment {
    pub fn new(value: &str) -> Self {
        RawFragment {
            value: value.to_string(),
        }
    }
}

impl TranslationFragmentable for RawFragment {
    fn render(self, _meta: &mut TranslateMetadata) -> String {
        self.value
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Raw(self)
    }
}
