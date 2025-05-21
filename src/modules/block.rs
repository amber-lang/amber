use std::collections::VecDeque;
use std::ops::Index;

use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use super::statement::stmt::Statement;

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub should_indent: bool,
    pub needs_noop: bool,
    pub is_conditional: bool,
}

impl Block {
    // Get whether this block is empty
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }


    pub fn with_condition(mut self) -> Self {
        self.is_conditional = true;
        self
    }

    // Push a parsed statement into the block
    pub fn push_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    pub fn with_needs_noop(mut self) -> Self {
        self.needs_noop = true;
        self
    }

    pub fn with_no_indent(mut self) -> Self {
        self.should_indent = false;
        self
    }
}

impl SyntaxModule<ParserMetadata> for Block {
    syntax_name!("Block");

    fn new() -> Self {
        Block {
            statements: vec![],
            should_indent: true,
            needs_noop: false,
            is_conditional: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        meta.with_push_scope(|meta| {
            while let Some(token) = meta.get_current_token() {
                // Handle the end of line or command
                if ["\n", ";"].contains(&token.word.as_str()) {
                    meta.increment_index();
                    continue;
                }
                // Handle block end
                else if token.word == "}" {
                    break;
                }
                let mut statement = Statement::new();
                if let Err(failure) = statement.parse(meta) {
                    return match failure {
                        Failure::Quiet(pos) => error_pos!(meta, pos, "Unexpected token"),
                        Failure::Loud(err) => return Err(Failure::Loud(err))
                    }
                }
                self.statements.push(statement);
            }
            Ok(())
        })
    }
}

impl TranslateModule for Block {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Save the current statement queue and create a new one
        let mut new_queue = VecDeque::new();
        std::mem::swap(&mut meta.stmt_queue, &mut new_queue);
        let result = {
            let mut statements = vec![];
            for statement in &self.statements {
                let statement = statement.translate(meta);
                statements.extend(meta.stmt_queue.drain(..));
                statements.push(statement);
            }
            BlockFragment::new(statements, self.should_indent)
                .with_needs_noop(self.needs_noop)
                .with_condition(self.is_conditional)
                .to_frag()
        };
        // Restore the old statement queue
        std::mem::swap(&mut meta.stmt_queue, &mut new_queue);
        result
    }
}

impl DocumentationModule for Block {
    fn document(&self, meta: &ParserMetadata) -> String {
        let indices = self.statements.iter()
            .enumerate()
            .map(|(index, statement)| (index, statement.get_docs_item_name()))
            .filter_map(|(index, name)| name.map(|n| (n, index)))
            .sorted()
            .collect::<Vec<_>>();
        indices.iter()
            .map(|(_, index)| self.statements.index(*index))
            .map(|statement| statement.document(meta))
            .collect::<Vec<_>>()
            .join("")
    }
}
