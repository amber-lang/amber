use super::fragment::{TranslationFragment, TranslationFragmentable};
use crate::utils::TranslateMetadata;

/// This module represents an eval string fragment.
/// Inside of an eval string, translation fragments are properly escaped.

#[derive(Debug, Clone)]
pub struct EvalFragment {
    pub fragment: Box<TranslationFragment>,
    pub toggle: bool,
}

impl EvalFragment {
    pub fn new(fragment: TranslationFragment, toggle: bool) -> Self {
        EvalFragment {
            fragment: Box::new(fragment),
            toggle,
        }
    }
}

impl TranslationFragmentable for EvalFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let prev = meta.eval_ctx;
        meta.eval_ctx = self.toggle;
        let result = self.fragment.to_string(meta);
        meta.eval_ctx = prev;

        if self.toggle {
            format!("eval \"{result}\"")
        } else {
            result
        }
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Eval(self)
    }
}
