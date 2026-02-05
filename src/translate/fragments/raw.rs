use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

/// This module represents a raw code fragment in Bash.
/// It is used to render code fragments that do not require any further processing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFragment {
    pub value: String,
}

impl From<String> for RawFragment {
    fn from(value: String) -> Self {
        RawFragment { value }
    }
}

impl RawFragment {
    pub fn new(value: &str) -> Self {
        RawFragment {
            value: value.to_string(),
        }
    }
}

impl FragmentRenderable for RawFragment {
    fn to_string(self, _meta: &mut TranslateMetadata) -> String {
        self.value
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Raw(self)
    }
}
