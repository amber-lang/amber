use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone)]
pub struct Return {
    pub expr: Expr
}

impl Typed for Return {
    fn get_type(&self) -> Type {
        self.expr.get_type()
    }
}

impl SyntaxModule<ParserMetadata> for Return {
    syntax_name!("Return");

    fn new() -> Self {
        Return {
            expr: Expr::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        token(meta, "return")?;
        if !meta.context.is_fun_ctx {
            return error!(meta, tok => {
                message: "Return statement outside of function",
                comment: "Return statements can only be used inside of functions"
            });
        }
        syntax(meta, &mut self.expr)?;
        match meta.context.fun_ret_type.as_ref() {
            Some(ret_type) => if ret_type != &self.expr.get_type() {
                return error!(meta, tok => {
                    message: "Return type does not match function return type",
                    comment: format!("Given type: {}, expected type: {}", self.expr.get_type(), ret_type)
                });
            },
            None => {
                meta.context.fun_ret_type = Some(self.expr.get_type());
            }
        }
        Ok(())
    }
}

impl TranslateModule for Return {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let (name, id, variant) = meta.fun_name.clone().expect("Function name not set");
        let result = self.expr.translate_eval(meta, false);
        let result = matches!(self.expr.get_type(), Type::Array(_))
            .then(|| format!("({result})"))
            .unwrap_or(result);
        meta.stmt_queue.push_back(format!("__AMBER_FUN_{name}{id}_v{variant}={result}"));
        "return 0".to_string()
    }
}

impl DocumentationModule for Return {
    fn document(&self) -> String {
        "".to_string()
    }
}
