use crate::translate::fragments::fragment::TranslationFragmentable;
use crate::{translate::fragments::fragment::TranslationFragment, utils::TranslateMetadata};

/// This module represents a Bash code fragment that is composed of multiple parts.
/// The sole purpose of this module is to bundle multiple `TranslationFragment` instances into a single fragment.

#[derive(Debug, Clone)]
pub struct CompoundFragment {
    fragments: Vec<TranslationFragment>,
}

impl CompoundFragment {
    pub fn new(frags: Vec<TranslationFragment>) -> Self {
        CompoundFragment { fragments: frags }
    }

    pub fn is_empty_logic(&self) -> bool {
        self.fragments.iter().all(|fragment| fragment.is_empty_logic())
    }
}

impl TranslationFragmentable for CompoundFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let mut result = String::new();
        for fragment in self.fragments {
            result.push_str(&fragment.to_string(meta));
        }
        result
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Compound(self)
    }
}
