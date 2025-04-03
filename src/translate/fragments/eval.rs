use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

/// This module represents an eval string fragment.
/// Inside of an eval string, translation fragments are properly escaped.
#[derive(Debug, Clone)]
pub struct EvalFragment {
    pub fragment: Box<FragmentKind>,
    pub eval_ctx: bool,
}

impl EvalFragment {
    pub fn new(fragment: FragmentKind, eval_ctx: bool) -> Self {
        EvalFragment {
            fragment: Box::new(fragment),
            eval_ctx,
        }
    }
}

impl FragmentRenderable for EvalFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let prev = meta.eval_ctx;
        meta.eval_ctx = self.eval_ctx;
        let result = self.fragment.to_string(meta);
        meta.eval_ctx = prev;

        if self.eval_ctx {
            format!("eval \"{result}\"")
        } else {
            result
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Eval(self)
    }
}
