use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Exit {
    code: Expr
}

impl SyntaxModule<ParserMetadata> for Exit {
    syntax_name!("Exit");

    fn new() -> Self {
        Exit {
            code: Expr::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "exit")?;
        syntax(meta, &mut self.code)?;
        let code_type = self.code.get_type();
        if code_type != Type::Num {
            let position = self.code.get_position(meta);
            return error_pos!(meta, position => {
                message: "Builtin function `exit` can only be used with values of type Num",
                comment: format!("Given type: {}, expected type: {}", code_type, Type::Num)
            });
        }
        Ok(())
    }
}

impl TranslateModule for Exit {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let code = self.code.translate(meta);
        format!("exit {}", code)
    }
}

impl DocumentationModule for Exit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
