use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Exit {
    code: Option<Expr>,
}

impl SyntaxModule<ParserMetadata> for Exit {
    syntax_name!("Exit");

    fn new() -> Self {
        Exit { code: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let position = meta.get_index();
        token(meta, "exit")?;

        if token(meta, "(").is_ok() {
            let mut code_expr = Expr::new();
            syntax(meta, &mut code_expr)?;
            self.code = Some(code_expr);
            token(meta, ")")?;
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling a builtin without parentheses is deprecated");
            meta.add_message(warning);

            let mut code_expr = Expr::new();
            if syntax(meta, &mut code_expr).is_ok() {
                self.code = Some(code_expr);
            }
        }
        Ok(())
    }
}

impl TypeCheckModule for Exit {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Some(ref mut code_expr) = self.code {
            code_expr.typecheck(meta)?;

            let code_type = code_expr.get_type();
            if code_type != Type::Int {
                let position = code_expr.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `exit` can only be used with values of type Int",
                    comment: format!("Given type: {}, expected type: {}", code_type, Type::Int)
                });
            }
        }
        Ok(())
    }
}

impl TranslateModule for Exit {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let exit_code = self.code.as_ref()
            .map(|expr| expr.translate(meta))
            .unwrap_or(fragments!("0"));
        fragments!("exit ", exit_code)
    }
}

impl DocumentationModule for Exit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
