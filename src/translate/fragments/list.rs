use crate::utils::TranslateMetadata;

use super::fragment::{TranslationFragment, TranslationFragmentable};

/// Represents a list of fragments that can be separated by a given separator.

#[derive(Debug, Clone)]
pub struct ListFragment {
    pub values: Vec<TranslationFragment>,
    pub separator: String,
}

impl ListFragment {
    pub fn new(value: Vec<TranslationFragment>, separator: &str) -> Self {
        ListFragment {
            values: value,
            separator: separator.to_string(),
        }
    }

    pub fn is_empty_logic(&self) -> bool {
        self.values.iter().all(|fragment| fragment.is_empty_logic())
    }
}

impl TranslationFragmentable for ListFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        self.values.into_iter().map(|x| x.to_string(meta)).collect::<Vec<String>>().join(&self.separator)
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::List(self)
    }
}
