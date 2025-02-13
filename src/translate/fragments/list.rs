use crate::utils::TranslateMetadata;

use super::fragment::{TranslationFragment, TranslationFragmentable};

#[derive(Debug, Clone)]
pub struct ListFragment {
    pub value: Vec<TranslationFragment>,
    pub separator: String,
}

impl ListFragment {
    pub fn new(value: Vec<TranslationFragment>, separator: &str) -> Self {
        ListFragment {
            value,
            separator: separator.to_string(),
        }
    }
}

impl TranslationFragmentable for ListFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        self.value.into_iter().map(|x| x.render(meta)).collect::<Vec<String>>().join(&self.separator)
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::List(self)
    }
}
