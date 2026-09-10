use crate::fragments;
use crate::modules::block::Block;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::prelude::*;
use crate::modules::statement::comment::Comment;
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

use std::collections::HashMap;
use crate::modules::expression::BoolAnalysis;

#[derive(Debug, Clone)]
struct IfChainBranch {
    comments: Vec<Comment>,
    cond: Expr,
    cfa: BoolAnalysis,
    block: Block,
}

impl IfChainBranch {
    pub fn new(comments: Vec<Comment>, cond: Expr, block: Block) -> Self {
        IfChainBranch { comments, cond, cfa: BoolAnalysis::default(), block }
    }

    pub fn with_cfa(mut self, cfa: BoolAnalysis) -> Self {
        self.cfa = cfa;
        self
    }
}

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "if"]
#[kind = "stmt"]
pub struct IfChain {
    cond_blocks: Vec<IfChainBranch>,
    false_block: Option<IfChainBranch>,
}

crate::impl_documentation_noop!(IfChain);

impl IfChain {
    pub fn terminates_control_flow(&self) -> bool {
        let mut all_previously_terminated = true;
        for branch in &self.cond_blocks {
            if branch.cfa.known_value == Some(true) {
                return all_previously_terminated && branch.block.terminates_control_flow();
            }
            all_previously_terminated &= branch.block.terminates_control_flow();
        }
        match (&self.false_block, all_previously_terminated) {
            (Some(branch), true) => branch.block.terminates_control_flow(),
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
                let mut else_block = Block::new().with_needs_noop().with_condition();
                syntax(meta, &mut else_block)?;
                self.false_block = Some(IfChainBranch::new(comments, cond, else_block));
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

            self.cond_blocks.push(IfChainBranch::new(comments, cond, block));
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
        let mut first_true_depends_on_target = false;
        let mut accumulated_neg_facts = HashMap::new();

        for mut branch in old_chain {
            for comment in branch.comments.iter_mut() {
                comment.typecheck(meta)?;
            }
            // Typecheck condition with accumulated negative facts
            meta.with_narrowed_scope(accumulated_neg_facts.clone(), |meta| branch.cond.typecheck(meta))?;
            let pos = branch.cond.get_position();

            if chain_deadcode {
                if !first_true_depends_on_target {
                    Self::warn_dead_code(
                        meta,
                        pos,
                        "Condition is unreachable, previous condition is always true",
                    );
                }
                continue;
            }

            let cfa = branch.cond.analyze_control_flow();
            match (cfa.known_value, cfa.has_side_effects) {
                (Some(true), false) => {
                    let (facts, _) = branch.cond.extract_facts();
                    // Merge accumulated negative facts with current positive facts for the block
                    let mut block_facts = accumulated_neg_facts.clone();
                    block_facts.extend(facts);

                    meta.with_narrowed_scope(block_facts, |meta| branch.block.typecheck(meta))?;
                    new_chain.push(branch.with_cfa(cfa));
                    chain_deadcode = true;
                    first_true_pos = Some(pos);
                    first_true_depends_on_target = cfa.depends_on_target;
                }
                (Some(false), false) => {
                    if !cfa.depends_on_target {
                        Self::warn_dead_code(
                            meta,
                            pos,
                            "Condition is always false, block will never execute",
                        );
                    }
                }
                _ => {
                    let (facts, neg_facts) = branch.cond.extract_facts();
                    // Merge accumulated negative facts with current positive facts for the block
                    let mut block_facts = accumulated_neg_facts.clone();
                    block_facts.extend(facts);

                    meta.with_narrowed_scope(block_facts, |meta| branch.block.typecheck(meta))?;
                    // Add current negative facts to the accumulated set for next branches
                    accumulated_neg_facts.extend(neg_facts);

                    new_chain.push(branch.with_cfa(cfa));
                }
            }
        }

        self.cond_blocks = new_chain;

        if chain_deadcode {
            if self.false_block.is_some() && !first_true_depends_on_target {
                if let Some(pos) = first_true_pos {
                    Self::warn_dead_code(
                        meta,
                        pos,
                        "Condition is always true, 'else' block will never execute",
                    );
                }
            }
            self.false_block = None;
        } else if let Some(branch) = &mut self.false_block {
            for comment in &mut branch.comments {
                comment.typecheck(meta)?;
            }
            meta.with_narrowed_scope(accumulated_neg_facts, |meta| branch.block.typecheck(meta))?;
        }

        Ok(())
    }
}

impl TranslateModule for IfChain {
  fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
      if self.cond_blocks.is_empty() {
          if let Some(false_branch) = &self.false_block {
              return false_branch.block.translate(meta);
          }
          return FragmentKind::Empty;
      }

      // In case of when only the first condition is true, we can just leave the truth block without any condition
      if let Some(first_branch) = self.cond_blocks.first() {
          let cfa = first_branch.cfa;
          if cfa.known_value == Some(true) && !cfa.has_side_effects {
              return first_branch.block.translate(meta);
          }
      }

      let mut result = vec![];
      let mut is_first = true;
      for branch in self.cond_blocks.iter() {
          for comment in &branch.comments {
              result.push(comment.translate(meta));
          }
          let condition = branch.cond.translate(meta)
              .with_quotes(
                  matches!(branch.cond.value, Some(ExprType::Text(_)))
              )
              .with_condition(true);

          if is_first {
              result.push(fragments!("if ", condition, "; then"));
              result.push(branch.block.translate(meta));
              is_first = false;
          } else {
              result.push(fragments!("elif ", condition, "; then"));
              result.push(branch.block.translate(meta));
          }
      }
      if let Some(false_branch) = &self.false_block {
          for comment in &false_branch.comments {
              result.push(comment.translate(meta));
          }
          result.push(fragments!("else"));
          result.push(false_branch.block.translate(meta));
      }
      result.push(fragments!("fi"));
      BlockFragment::new(result, false).to_frag()
  }
}
