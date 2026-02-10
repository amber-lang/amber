use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;
use std::mem;

/// Renders blocks of statements in Bash code.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockFragment {
    pub statements: Vec<FragmentKind>,
    pub increase_indent: bool,
    pub needs_noop: bool,
    pub is_conditional: bool,
}

impl BlockFragment {
    pub fn new(statements: Vec<FragmentKind>, increase_indent: bool) -> Self {
        BlockFragment {
            statements,
            increase_indent,
            needs_noop: false,
            is_conditional: false,
        }
    }

    pub fn with_needs_noop(mut self, needs_noop: bool) -> Self {
        self.needs_noop = needs_noop;
        self
    }

    pub fn with_condition(mut self, is_conditional: bool) -> Self {
        self.is_conditional = is_conditional;
        self
    }

    pub fn append(&mut self, statement: FragmentKind) {
        self.statements.push(statement);
    }

    pub fn is_empty_logic(&self) -> bool {
        self.statements
            .iter()
            .all(|fragment| fragment.is_empty_logic())
    }
}

impl FragmentRenderable for BlockFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let empty_logic = self.is_empty_logic();
        if self.increase_indent {
            meta.increase_indent();
        }
        let mut result = vec![];
        for statement in self.statements {
            match statement {
                FragmentKind::Empty => (),
                FragmentKind::Block(block) => {
                    let rendered = block.to_string(meta);
                    if !rendered.is_empty() {
                        result.push(rendered);
                    }
                }
                _ => {
                    let statement = statement.to_string(meta);
                    for stmt in mem::take(&mut meta.stmt_queue) {
                        result.push(meta.gen_indent() + &stmt.to_string(meta));
                    }
                    result.push(meta.gen_indent() + &statement);
                }
            }
        }
        if empty_logic && self.needs_noop {
            result.push(meta.gen_indent() + ":");
        }
        if self.increase_indent {
            meta.decrease_indent();
        }
        result.join("\n")
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Block(self)
    }
}
