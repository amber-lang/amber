use super::fragment::{FragmentKind, FragmentRenderable};
use crate::{translate::compare::create_bool_comparison, utils::TranslateMetadata};

/// This module represents a raw code fragment in Bash.
/// It is used to render code fragments that do not require any further processing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFragment {
    pub value: String,
    pub with_condition: bool
}

impl From<String> for RawFragment {
    fn from(value: String) -> Self {
        RawFragment { value, with_condition: false }
    }
}

impl RawFragment {
    pub fn new(value: &str) -> Self {
        RawFragment {
            value: value.to_string(),
            with_condition: false
        }
    }
    pub fn with_condition(mut self, condition: bool) -> Self {
        self.with_condition = condition;
        self
    }
}

impl FragmentRenderable for RawFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        if self.with_condition {
            create_bool_comparison(meta, self.value)
        } else {
            self.value
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Raw(self)
    }
}
