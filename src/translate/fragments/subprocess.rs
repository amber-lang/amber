use super::fragment::{FragmentKind, FragmentRenderable};
use crate::{translate::compare::create_bool_comparison, utils::TranslateMetadata};

// Creates a subprocess fragment that is correctly escaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprocessFragment {
    pub fragment: Box<FragmentKind>,
    pub quoted: bool,
    pub with_condition: bool
}

impl SubprocessFragment {
    pub fn new(fragment: FragmentKind) -> Self {
        SubprocessFragment {
            fragment: Box::new(fragment),
            quoted: true,
            with_condition: false
        }
    }

    pub fn with_condition(mut self, condition: bool) -> Self {
        self.with_condition = condition;
        self
    }

    pub fn with_quotes(mut self, quoted: bool) -> Self {
        self.quoted = quoted;
        self
    }
}

impl FragmentRenderable for SubprocessFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let result = self.fragment.to_string(meta);
        let dollar = meta.gen_dollar();
        let quote = if self.quoted { meta.gen_quote() } else { "" };

        let result = if meta.eval_ctx {
            format!("{quote}{dollar}(eval \"{result}\"){quote}")
        } else {
            format!("{quote}{dollar}({result}){quote}")
        };

        if self.with_condition {
            create_bool_comparison(meta, result)
        } else {
            result
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Subprocess(self)
    }
}
