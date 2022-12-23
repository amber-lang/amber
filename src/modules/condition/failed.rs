use heraclitus_compiler::prelude::*;
use crate::modules::block::Block;
use crate::modules::statement::stmt::Statement;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Failed {
    is_parsed: bool,
    is_question_mark: bool,
    is_main: bool,
    block: Box<Block>
}

impl SyntaxModule<ParserMetadata> for Failed {
    syntax_name!("Failed Expression");

    fn new() -> Self {
        Failed {
            is_parsed: false,
            is_question_mark: false,
            is_main: false,
            block: Box::new(Block::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        if let Ok(_) = token(meta, "?") {
            if !meta.context.is_fun_ctx && !meta.context.is_main_ctx {
                return error!(meta, tok, "The '?' operator can only be used in the main block or function body")
            }
            self.is_question_mark = true;
            self.is_main = meta.context.is_main_ctx;
            self.is_parsed = true;
            return Ok(())
        }
        token(meta, "failed")?;
        match token(meta, "{") {
            Ok(_) => {
                syntax(meta, &mut *self.block)?;
                token(meta, "}")?;
            },
            Err(_) => {
                token(meta, "=>")?;
                let mut statement = Statement::new();
                syntax(meta, &mut statement)?;
                self.block.push_statement(statement);
            }
        }
        self.is_main = meta.context.is_main_ctx;
        self.is_parsed = true;
        Ok(())
    }
}

impl TranslateModule for Failed {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        if self.is_parsed {
            let block = self.block.translate(meta);
            let ret = self.is_main
                .then(|| "exit $?")
                .unwrap_or("return $?");
            // the condition of '$?' clears the status code thus we need to store it in a variable
            if self.is_question_mark {
                vec![
                    "__AMBER_STATUS=$?;",
                    "if [ $__AMBER_STATUS != 0 ]; then",
                    &format!("$(exit $__AMBER_STATUS)"),
                    ret,
                    "fi"
                ].join("\n")
            } else {
                vec![
                    "__AMBER_STATUS=$?;",
                    "if [ $__AMBER_STATUS != 0 ]; then",
                    &format!("$(exit $__AMBER_STATUS)"),
                    &block,
                    "fi"
                ].join("\n")
            }
        } else {
            String::new()
        }
    }
}