use std::ops::Index;

use super::statement::stmt::Statement;
use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub should_indent: bool,
    pub needs_noop: bool,
    pub parses_syntax: bool,
    pub is_conditional: bool,
}

impl Block {
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    pub fn terminates_control_flow(&self) -> bool {
        self.flow().terminates
    }

    pub fn flow(&self) -> FlowSummary {
        let mut acc = FlowSummary::default();
        for stmt in &self.statements {
            acc = acc.then(|| stmt.flow());
            if acc.terminates {
                return acc;
            }
        }
        acc
    }

    pub fn set_no_indent(&mut self) {
        self.should_indent = false;
    }

    pub fn with_condition(mut self) -> Self {
        self.is_conditional = true;
        self
    }

    pub fn with_needs_noop(mut self) -> Self {
        self.needs_noop = true;
        self
    }

    pub fn with_no_indent(mut self) -> Self {
        self.should_indent = false;
        self
    }

    pub fn with_no_syntax(mut self) -> Self {
        self.parses_syntax = false;
        self
    }
}

impl ControlFlow for Block {
    fn flow(&self) -> FlowSummary {
        self.flow()
    }
}

impl SyntaxModule<ParserMetadata> for Block {
    syntax_name!("Block");

    fn new() -> Self {
        Block {
            statements: vec![],
            should_indent: true,
            parses_syntax: true,
            needs_noop: false,
            is_conditional: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let is_single_line = if self.parses_syntax {
            let parsed_word = token_by(meta, |word| [":", "{"].contains(&word.as_str()))?;
            parsed_word == ":"
        } else {
            false
        };

        while meta.get_current_token().is_some() {
            // Handle the end of line
            if token(meta, "\n").is_ok() {
                continue;
            }
            // Handle block end
            if !is_single_line && self.parses_syntax && token(meta, "}").is_ok() {
                break;
            }
            let mut statement = Statement::new();
            if let Err(failure) = statement.parse(meta) {
                return match failure {
                    Failure::Quiet(pos) => error_pos!(meta, pos, "Unexpected token"),
                    Failure::Loud(err) => return Err(Failure::Loud(err)),
                };
            }
            self.statements.push(statement);
            // Handle the semicolon
            token(meta, ";").ok();
            // Handle single line
            if is_single_line {
                break;
            }
        }
        Ok(())
    }
}

impl TypeCheckModule for Block {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let no_scope_exists = meta.context.scopes.is_empty();
        meta.with_push_scope(self.parses_syntax || no_scope_exists, |meta| {
            // Type check all statements in the block
            for statement in &mut self.statements {
                statement.typecheck(meta)?;
            }
            Ok(())
        })
    }
}

impl TranslateModule for Block {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let mut statements = vec![];
        for statement in &self.statements {
            let (stmts, translated) = meta.capture_stmt_queue(|meta| statement.translate(meta));
            statements.extend(stmts);
            statements.push(translated);
            if statement.terminates_control_flow() {
                break;
            }
        }
        BlockFragment::new(statements, self.should_indent)
            .with_needs_noop(self.needs_noop)
            .with_condition(self.is_conditional)
            .to_frag()
    }
}

impl DocumentationModule for Block {
    fn document(&self, meta: &ParserMetadata) -> String {
        let indices = self
            .statements
            .iter()
            .enumerate()
            .map(|(index, statement)| (index, statement.get_docs_item_name()))
            .filter_map(|(index, name)| name.map(|n| (n, index)))
            .sorted()
            .collect::<Vec<_>>();
        indices
            .iter()
            .map(|(_, index)| self.statements.index(*index))
            .map(|statement| statement.document(meta))
            .collect::<Vec<_>>()
            .join("")
    }
}
