use crate::translate::fragments::fragment::TranslationFragmentable;
use crate::{translate::fragments::fragment::TranslationFragment, utils::TranslateMetadata};

#[derive(Debug, Clone)]
pub struct CompoundFragment {
    fragments: Vec<TranslationFragment>,
}

impl CompoundFragment {
    pub fn new(frags: Vec<TranslationFragment>) -> Self {
        CompoundFragment { fragments: frags }
    }
}

impl TranslationFragmentable for CompoundFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        let mut result = String::new();
        for fragment in self.fragments {
            result.push_str(&fragment.render(meta));
        }
        result
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Compound(self)
    }
}
