use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ListFragmentSeparator {
    Space,
    Empty,
}

/// Represents a list of fragments that can be separated by a given separator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFragment {
    pub values: Vec<FragmentKind>,
    separator: ListFragmentSeparator,
}

impl ListFragment {
    pub fn new(value: Vec<FragmentKind>) -> Self {
        ListFragment {
            values: value,
            separator: ListFragmentSeparator::Empty,
        }
    }

    pub fn with_spaces(mut self) -> Self {
        self.separator = ListFragmentSeparator::Space;
        self
    }

    pub fn is_empty_logic(&self) -> bool {
        self.values.iter().all(|fragment| fragment.is_empty_logic())
    }
}

impl FragmentRenderable for ListFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let sep: &'static str = match self.separator {
            ListFragmentSeparator::Space => " ",
            ListFragmentSeparator::Empty => "",
        };
        self.values
            .into_iter()
            .filter(|x| !matches!(x, FragmentKind::Empty))
            .map(|x| x.to_string(meta))
            .collect::<Vec<String>>()
            .join(sep)
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::List(self)
    }
}
