use std::mem;
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

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

impl TranslationFragmentable for BlockFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        if self.is_empty() && self.increase_indent {
            return ":".to_string()
        }
        if self.increase_indent {
            meta.increase_indent();
        }
        let mut result = vec![];
        for statement in self.statements {
            match statement {
                TranslationFragment::Empty => {
                    continue
                }
                TranslationFragment::Block(block) => {
                    result.push(block.render(meta));
                }
                _ => {
                    let statement = statement.render(meta);
                    for stmt in mem::take(&mut meta.stmt_queue) {
                        result.push(meta.gen_indent() + &stmt.render(meta));
                    }
                    result.push(meta.gen_indent() + &statement);
                }
            }
        }
        if self.increase_indent {
            meta.decrease_indent();
        }
        result.join("\n")
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Block(self)
    }
}
