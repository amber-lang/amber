use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::utils::cc_flags::{CCFlags, get_ccflag_name};
use crate::modules::statement::stmt::{Statement, StmtType};
use crate::modules::block::Block;

#[derive(Debug, Clone)]
pub struct IfCondition {
    expr: Box<Expr>,
    true_block: Option<Box<Block>>,
    false_block: Option<Box<Block>>,
}

impl IfCondition {
    fn prevent_not_using_if_chain(&self, meta: &mut ParserMetadata, statement: &Statement, tok: Option<Token>) -> Result<(), Failure> {
        let is_not_if_chain = matches!(statement.value.as_ref().unwrap(), StmtType::IfCondition(_) | StmtType::IfChain(_));
        if is_not_if_chain && !meta.context.cc_flags.contains(&CCFlags::AllowNestedIfElse) {
            let flag_name = get_ccflag_name(CCFlags::AllowNestedIfElse);
            let message = Message::new_warn_at_token(meta, tok)
                .message("You should use if-chain instead of nested if else statements")
                .comment(format!("To suppress this warning, use '{flag_name}' compiler flag"));
            meta.add_message(message);
        }
        Ok(())
    }

    fn warn_dead_code(meta: &mut ParserMetadata, pos: PositionInfo, is_true_branch: bool) {
        if meta.context.cc_flags.contains(&CCFlags::AllowDeadCode) {
            return;
        }
        let flag_name = get_ccflag_name(CCFlags::AllowDeadCode);
        let branch = if is_true_branch { "if" } else { "else" };
        let message = Message::new_warn_at_position(meta, pos)
            .message(format!("Condition is always {}, '{branch}' block will never execute", !is_true_branch))
            .comment(format!("To suppress this warning, use '{flag_name}' compiler flag"));
        meta.add_message(message);
    }
}

impl SyntaxModule<ParserMetadata> for IfCondition {
    syntax_name!("If Condition");

    fn new() -> Self {
        IfCondition {
            expr: Box::new(Expr::new()),
            true_block: None,
            false_block: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "if")?;
        // Parse expression
        syntax(meta, &mut *self.expr)?;
        // Parse true block
        let mut true_block = Box::new(Block::new().with_needs_noop().with_condition());
        syntax(meta, &mut *true_block)?;
        self.true_block = Some(true_block);
        // Parse false block
        if token(meta, "else").is_ok() {
            let mut false_block = Box::new(Block::new().with_needs_noop().with_condition());
            let tok = meta.get_current_token();
            syntax(meta, &mut *false_block)?;

            // Check if the statement is using if chain syntax sugar
            if false_block.statements.len() == 1 {
                if let Some(statement) = false_block.statements.first() {
                    self.prevent_not_using_if_chain(meta, statement, tok)?;
                }
            }
            self.false_block = Some(false_block);
        }
        Ok(())
    }
}

impl TypeCheckModule for IfCondition {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        match self.expr.analyze_control_flow() {
            Some(true) => {
                // Condition always true
                if self.false_block.is_some() {
                    let pos = self.expr.get_position();
                    Self::warn_dead_code(meta, pos, false);
                }
                self.false_block = None;
                 
                let (true_facts, _) = self.expr.extract_facts();
                if let Some(true_block) = &mut self.true_block {
                    meta.with_narrowed_scope(true_facts, |meta| {
                        true_block.typecheck(meta)
                    })?;
                }
            },
            Some(false) => {
                // Condition always false
                let pos = self.expr.get_position();
                Self::warn_dead_code(meta, pos, true);
                self.true_block = None;
                 
                let (_, false_facts) = self.expr.extract_facts();
                if let Some(false_block) = &mut self.false_block {
                    meta.with_narrowed_scope(false_facts, |meta| {
                        false_block.typecheck(meta)
                    })?;
                }
            },
            None => {
                let (true_facts, false_facts) = self.expr.extract_facts();
                if let Some(true_block) = &mut self.true_block {
                    meta.with_narrowed_scope(true_facts, |meta| {
                        true_block.typecheck(meta)
                    })?;
                }

                if let Some(false_block) = &mut self.false_block {
                    meta.with_narrowed_scope(false_facts, |meta| {
                        false_block.typecheck(meta)
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl TranslateModule for IfCondition {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        match self.expr.analyze_control_flow() {
            Some(true) => {
                self.true_block.as_ref()
                    .map(|b| b.translate(meta))
                    .unwrap_or(FragmentKind::Empty)
            },
            Some(false) => {
                self.false_block.as_ref()
                    .map(|b| b.translate(meta))
                    .unwrap_or(FragmentKind::Empty)
            },
            None => {
                let mut result = vec![];
                result.push(fragments!("if [ ", self.expr.translate(meta), " != 0 ]; then"));
                if let Some(true_block) = &self.true_block {
                    result.push(true_block.translate(meta));
                }
                if let Some(false_block) = &self.false_block {
                    result.push(fragments!("else"));
                    result.push(false_block.translate(meta));
                }
                result.push(fragments!("fi"));
                BlockFragment::new(result, false).to_frag()
            }
        }
    }
}

impl DocumentationModule for IfCondition {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
