use super::fragment::{TranslationFragment, TranslationFragmentable};
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone)]
pub struct BlockFragment {
    pub statements: Vec<TranslationFragment>,
    pub increase_indent: bool,
}

impl BlockFragment {
    pub fn new(statements: Vec<TranslationFragment>, increase_indent: bool) -> Self {
        BlockFragment {
            statements,
            increase_indent,
        }
    }

    pub fn append(&mut self, statement: TranslationFragment) {
        self.statements.push(statement);
    }
}

impl TranslationFragmentable for BlockFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        if self.increase_indent {
            meta.increase_indent();
        }
        let mut result = String::new();
        for statement in self.statements {
            if let TranslationFragment::Empty = statement {
                continue;
            }
            result.push_str(&meta.gen_indent());
            result.push_str(&statement.render(meta));
        }
        if self.increase_indent {
            meta.decrease_indent();
        }
        result
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Block(self)
    }
}
