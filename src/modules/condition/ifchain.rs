use crate::fragments;
use crate::modules::block::Block;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::statement::comment::Comment;
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use heraclitus_compiler::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IfChain {
    cond_blocks: Vec<(Vec<Comment>, Expr, Block)>,
    false_block: Option<(Vec<Comment>, Box<Block>)>,
}

impl IfChain {
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
                    return error!(
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

        // In case of when only the first condition is true, we can just leave the truth block without any condition
        if let Some((_, first_cond, first_block)) = self.cond_blocks.first() {
            if first_cond.analyze_control_flow() == Some(true) {
                return first_block.translate(meta);
            }
        }

        let mut result = vec![];
        let mut is_first = true;
        for (comments, cond, block) in self.cond_blocks.iter() {
            for comment in comments {
                result.push(comment.translate(meta));
            }
            if is_first {
                result.push(fragments!("if [ ", cond.translate(meta), " != 0 ]; then"));
                result.push(block.translate(meta));
                is_first = false;
            } else {
                result.push(fragments!("elif [ ", cond.translate(meta), " != 0 ]; then"));
                result.push(block.translate(meta));
            }
        }
        if let Some((comments, false_block)) = &self.false_block {
            for comment in comments {
                result.push(comment.translate(meta));
            }
            result.push(fragments!("else"));
            result.push(false_block.translate(meta));
        }
        result.push(fragments!("fi"));
        BlockFragment::new(result, false).to_frag()
    }
}

impl DocumentationModule for IfChain {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
