use super::fragment::{TranslationFragment, TranslationFragmentable};
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone)]
pub struct SubprocessFragment {
    pub fragment: Box<TranslationFragment>,
}

impl SubprocessFragment {
    pub fn new(fragment: TranslationFragment) -> Self {
        SubprocessFragment {
            fragment: Box::new(fragment),
        }
    }
}

impl TranslationFragmentable for SubprocessFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        let result = self.fragment.render(meta);
        if meta.eval_ctx {
            format!("$(eval \"{}\")", result)
        } else {
            format!("$({})", result)
        }
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Subprocess(self)
    }
}
