use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::block::Block;
use crate::modules::statement::comment::Comment;

#[derive(Debug, Clone)]
pub struct IfChain {
    cond_blocks: Vec<(Vec<Comment>, Expr, Block)>,
    false_block: Option<(Vec<Comment>, Box<Block>)>
}

impl SyntaxModule<ParserMetadata> for IfChain {
    syntax_name!("If Condition");

    fn new() -> Self {
        IfChain {
            cond_blocks: vec![],
            false_block: None
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
                continue
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
                  return error!(meta, meta.get_current_token(), "Expected `else` condition to be the last in the if chain")?
                }
                return Ok(())
            }
            // Handle end of the if chain
            if token(meta, "}").is_ok() {
                return Ok(())
            }
            syntax(meta, &mut cond)?;
            syntax(meta, &mut block)?;

            self.cond_blocks.push((comments, cond, block));
        }
    }
}

impl TypeCheckModule for IfChain {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Type-check all condition-block pairs
        for (comments, cond, block) in &mut self.cond_blocks {
            for comment in comments {
                comment.typecheck(meta)?;
            }
            cond.typecheck(meta)?;
            block.typecheck(meta)?;
        }

        // Type-check the false block if it exists
        if let Some((comments, false_block)) = &mut self.false_block {
            for comment in comments {
                comment.typecheck(meta)?;
            }
            false_block.typecheck(meta)?;
        }

        Ok(())
    }
}

impl TranslateModule for IfChain {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
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
