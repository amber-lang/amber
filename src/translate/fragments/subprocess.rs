use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

// Creates a subprocess fragment that is correctly escaped.
#[derive(Debug, Clone)]
pub struct SubprocessFragment {
    pub fragment: Box<FragmentKind>,
    pub quoted: bool,
}

impl SubprocessFragment {
    pub fn new(fragment: FragmentKind) -> Self {
        SubprocessFragment {
            fragment: Box::new(fragment),
            quoted: true,
        }
    }

    pub fn set_quoted(mut self, quoted: bool) -> Self {
        self.quoted = quoted;
        self
    }
}

impl FragmentRenderable for SubprocessFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let result = self.fragment.to_string(meta);
        let quote = if self.quoted { meta.gen_quote() } else { "" };
        if meta.eval_ctx {
            format!("{quote}$(eval \"{}\"){quote}", result)
        } else {
            format!("{quote}$({}){quote}", result)
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Subprocess(self)
    }
}
