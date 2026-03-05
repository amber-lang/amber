use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

/// Represents a comment fragment in the translation process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentFragment {
    pub value: String,
}

impl CommentFragment {
    pub fn new(value: &str) -> Self {
        CommentFragment {
            value: value.to_string(),
        }
    }
}

impl FragmentRenderable for CommentFragment {
    fn to_string(self, _meta: &mut TranslateMetadata) -> String {
        format!("# {}", self.value)
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Comment(self)
    }
}

impl From<String> for CommentFragment {
    fn from(value: String) -> Self {
        Self::new(value.as_str())
    }
}
