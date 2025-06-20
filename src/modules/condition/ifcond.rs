use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::utils::cc_flags::{CCFlags, get_ccflag_name};
use crate::modules::statement::stmt::{Statement, StatementType};
use crate::modules::block::Block;

#[derive(Debug, Clone)]
pub struct IfCondition {
    expr: Box<Expr>,
    true_block: Box<Block>,
    false_block: Option<Box<Block>>,
}

impl IfCondition {
    fn prevent_not_using_if_chain(&self, meta: &mut ParserMetadata, statement: &Statement, tok: Option<Token>) -> Result<(), Failure> {
        let is_not_if_chain = matches!(statement.value.as_ref().unwrap(), StatementType::IfCondition(_) | StatementType::IfChain(_));
        if is_not_if_chain && !meta.context.cc_flags.contains(&CCFlags::AllowNestedIfElse) {
            let flag_name = get_ccflag_name(CCFlags::AllowNestedIfElse);
            // TODO: [A34] Add a comment pointing to the website documentation
            let message = Message::new_warn_at_token(meta, tok)
                .message("You should use if-chain instead of nested if else statements")
                .comment(format!("To suppress this warning, use '{flag_name}' compiler flag"));
            meta.add_message(message);
        }
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for IfCondition {
    syntax_name!("If Condition");

    fn new() -> Self {
        IfCondition {
            expr: Box::new(Expr::new()),
            true_block: Box::new(Block::new().with_needs_noop().with_condition()),
            false_block: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "if")?;
        // Parse expression
        syntax(meta, &mut *self.expr)?;
        // Parse true block
        match token(meta, "{") {
            Ok(_) => {
                syntax(meta, &mut *self.true_block)?;
                token(meta, "}")?;
            }
            Err(_) => {
                let mut statement = Statement::new();
                token(meta, ":")?;
                syntax(meta, &mut statement)?;
                self.true_block.push_statement(statement);
            }
        }
        // Parse false block
        if token(meta, "else").is_ok() {
            match token(meta, "{") {
                Ok(_) => {
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
                    token(meta, "}")?;
                }
                Err(_) => {
                    token(meta, ":")?;
                    let tok = meta.get_current_token();
                    let mut statement = Statement::new();
                    syntax(meta, &mut statement)?;
                    // Check if the statement is using if chain syntax sugar
                    self.prevent_not_using_if_chain(meta, &statement, tok)?;
                    self.false_block.get_or_insert(Box::new(Block::new())).push_statement(statement);
                }
            }
        }
        Ok(())
    }
}

impl TranslateModule for IfCondition {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let mut result = vec![];
        result.push(fragments!("if [ ", self.expr.translate(meta), " != 0 ]; then"));
        result.push(self.true_block.translate(meta));
        if let Some(false_block) = &self.false_block {
            result.push(fragments!("else"));
            result.push(false_block.translate(meta));
        }
        result.push(fragments!("fi"));
        BlockFragment::new(result, false).to_frag()
    }
}

impl DocumentationModule for IfCondition {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
