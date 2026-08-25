use crate::fragments;
use crate::modules::block::Block;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::prelude::*;
use crate::modules::statement::comment::Comment;
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

use std::collections::HashMap;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "if"]
#[kind = "stmt"]
pub struct IfChain {
    cond_blocks: Vec<(Vec<Comment>, Expr, Block)>,
    pub false_block: Option<(Vec<Comment>, Box<Block>)>,
}

crate::impl_documentation_noop!(IfChain);

impl IfChain {
    pub fn terminates_control_flow(&self) -> bool {
        for (_, cond, block) in &self.cond_blocks {
            if cond.analyze_control_flow() == Some(true) {
                return block.terminates_control_flow();
            }
        }
        let all_terminate = self
            .cond_blocks
            .iter()
            .all(|(_, _, block)| block.terminates_control_flow());
        match (&self.false_block, all_terminate) {
            (Some((_, false_block)), true) => false_block.terminates_control_flow(),
            _ => false,
        }
    }

    fn warn_dead_code(meta: &mut ParserMetadata, pos: PositionInfo, reason: &str) {
        if meta.context.cc_flags.contains(&CCFlags::AllowDeadCode) {
            return;
        }
        let flag_name = get_ccflag_name(CCFlags::AllowDeadCode);
        let message = Message::new_warn_at_position(meta, pos)
            .message(reason)
            .comment(format!(
                "To suppress this warning, use '{flag_name}' compiler flag"
            ));
        meta.add_message(message);
    }
}

impl SyntaxModule<ParserMetadata> for IfChain {
    syntax_name!("If Condition");

    fn new() -> Self {
        IfChain {
            cond_blocks: vec![],
            false_block: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "if")?;
        // Parse true block
        token(meta, "{")?;
        loop {
            let mut comments = vec![];
            let mut cond = Expr::new();
            let mut block = Block::new().with_needs_noop().with_condition();

            // Handle new lines
            if token_by(meta, |token| token.starts_with('\n')).is_ok() {
                continue;
            }

            // Handle comments
            loop {
                if meta
                    .get_current_token()
                    .is_some_and(|t| t.word.starts_with("//"))
                {
                    let mut comment = Comment::new();
                    syntax(meta, &mut comment)?;
                    comments.push(comment);

                    let _ = token_by(meta, |t| t.starts_with('\n'));
                } else {
                    break;
                }
            }

            // Handle else keyword
            if token(meta, "else").is_ok() {
                let mut false_block = Box::new(Block::new().with_needs_noop().with_condition());
                syntax(meta, &mut *false_block)?;
                self.false_block = Some((comments, false_block));
                if token(meta, "}").is_err() {
                    error!(
                        meta,
                        meta.get_current_token(),
                        "Expected `else` condition to be the last in the if chain"
                    )?;
                }
                return Ok(());
            }
            // Handle end of the if chain
            if token(meta, "}").is_ok() {
                return Ok(());
            }
            syntax(meta, &mut cond)?;
            syntax(meta, &mut block)?;

            self.cond_blocks.push((comments, cond, block));
        }
    }
}

impl TypeCheckModule for IfChain {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let old_chain = std::mem::take(&mut self.cond_blocks);
        let mut new_chain = Vec::new();
        let mut chain_deadcode = false;
        // Used for warning about unreachable conditions
        let mut first_true_pos: Option<PositionInfo> = None;
        let mut accumulated_neg_facts = HashMap::new();

        for (mut comments, mut cond, mut block) in old_chain {
            for comment in comments.iter_mut() {
                comment.typecheck(meta)?;
            }
            // Typecheck condition with accumulated negative facts
            meta.with_narrowed_scope(accumulated_neg_facts.clone(), |meta| cond.typecheck(meta))?;
            let pos = cond.get_position();

            if chain_deadcode {
                Self::warn_dead_code(
                    meta,
                    pos,
                    "Condition is unreachable, previous condition is always true",
                );
                continue;
            }

            match cond.analyze_control_flow() {
                Some(true) => {
                    let (facts, _) = cond.extract_facts();
                    // Merge accumulated negative facts with current positive facts for the block
                    let mut block_facts = accumulated_neg_facts.clone();
                    block_facts.extend(facts);

                    meta.with_narrowed_scope(block_facts, |meta| block.typecheck(meta))?;
                    new_chain.push((comments, cond, block));
                    chain_deadcode = true;
                    first_true_pos = Some(pos);
                }
                Some(false) => {
                    Self::warn_dead_code(
                        meta,
                        pos,
                        "Condition is always false, block will never execute",
                    );
                }
                None => {
                    let (facts, neg_facts) = cond.extract_facts();
                    // Merge accumulated negative facts with current positive facts for the block
                    let mut block_facts = accumulated_neg_facts.clone();
                    block_facts.extend(facts);

                    meta.with_narrowed_scope(block_facts, |meta| block.typecheck(meta))?;
                    // Add current negative facts to the accumulated set for next branches
                    accumulated_neg_facts.extend(neg_facts);

                    new_chain.push((comments, cond, block));
                }
            }
        }

        self.cond_blocks = new_chain;

        if chain_deadcode {
            if self.false_block.is_some() {
                if let Some(pos) = first_true_pos {
                    Self::warn_dead_code(
                        meta,
                        pos,
                        "Condition is always true, 'else' block will never execute",
                    );
                }
            }
            self.false_block = None;
        } else if let Some((comments, false_block)) = &mut self.false_block {
            for comment in comments {
                comment.typecheck(meta)?;
            }
            meta.with_narrowed_scope(accumulated_neg_facts, |meta| false_block.typecheck(meta))?;
        }

        Ok(())
    }
}

impl TranslateModule for IfChain {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.cond_blocks.is_empty() {
            if let Some((_, false_block)) = &self.false_block {
                return false_block.translate(meta);
            }
            return FragmentKind::Empty;
        }

        // Check if the first condition can be folded to a constant
        if let Some((_, first_cond, first_block)) = self.cond_blocks.first() {
            if first_cond.try_fold_bool_constant(meta) == Some(true) {
                return first_block.translate(meta);
            }
        }

        let mut result = vec![];
        let mut has_emitted_any_branch = false;
        let mut first_taken_block = None;
        
        for (comments, cond, block) in self.cond_blocks.iter() {
            for comment in comments {
                result.push(comment.translate(meta));
            }
            
            // Try to fold this condition to a constant
            if let Some(constant_value) = cond.try_fold_bool_constant(meta) {
                if constant_value {
                    // This branch is always taken - save it as the first taken block
                    if first_taken_block.is_none() {
                        first_taken_block = Some((comments, block));
                    }
                    // If we already have a first taken block, this is dead code (shouldn't happen)
                }
                // If false, skip this branch entirely
                continue;
            }
            
            // Non-constant condition - emit normally
            let condition = cond.translate(meta)
                .with_quotes(
                    matches!(cond.value, Some(ExprType::Text(_)))
                )
                .with_condition(true);
                
            if !has_emitted_any_branch {
                result.push(fragments!("if ", condition, "; then"));
                has_emitted_any_branch = true;
            } else {
                result.push(fragments!("elif ", condition, "; then"));
            }
            result.push(block.translate(meta));
        }
        
        // Handle the first taken constant-true block
        if let Some((comments, block)) = first_taken_block {
            if !has_emitted_any_branch {
                // This is the only taken branch, emit it without if/elif/fi
                for comment in comments {
                    result.push(comment.translate(meta));
                }
                result.push(block.translate(meta));
                // No fi needed since there's no if
            } else {
                // We already have if/elif, so this constant-true branch is dead code
                // (shouldn't happen in valid code, but ignore it)
            }
        }
        
        if let Some((comments, false_block)) = &self.false_block {
            for comment in comments {
                result.push(comment.translate(meta));
            }
            // Only emit else if we have an if/elif before it
            if has_emitted_any_branch {
                result.push(fragments!("else"));
                result.push(false_block.translate(meta));
                result.push(fragments!("fi"));
            } else if first_taken_block.is_none() {
                // No branches at all, just emit the else block
                result.push(false_block.translate(meta));
            }
            // If first_taken_block is Some and has_emitted_any_branch is false,
            // we already emitted the constant-true block without fi, so no else
        } else if has_emitted_any_branch {
            // No else block but we have if/elif
            result.push(fragments!("fi"));
        }
        // If first_taken_block is Some and has_emitted_any_branch is false,
        // we already emitted without fi, so nothing more to do
        
        BlockFragment::new(result, false).to_frag()
    }
}
