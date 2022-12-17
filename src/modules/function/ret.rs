use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone)]
pub struct Ret {
    pub expr: Expr
}

impl Typed for Ret {
    fn get_type(&self) -> Type {
        self.expr.get_type()
    }
}

impl SyntaxModule<ParserMetadata> for Ret {
    syntax_name!("Ret");

    fn new() -> Self {
        Ret {
            expr: Expr::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        token(meta, "ret")?;
        if !meta.context.is_fun_ctx {
            return error!(meta, tok,
                "Return statement outside of function",
                "Return statements can only be used inside of functions"
            );
        }
        syntax(meta, &mut self.expr)?;
        match meta.context.fun_ret_type.as_ref() {
            Some(ret_type) => if ret_type != &self.expr.get_type() {
                return error!(meta, tok,
                    "Return type does not match function return type",
                    format!("Given type: {}, expected type: {}", self.expr.get_type(), ret_type)
                );
            },
            None => {}
        }
        Ok(())
    }
}

impl TranslateModule for Ret {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let (name, id, variant) = meta.fun_name.clone().expect("Function name not set");
        let result = self.expr.translate(meta);
        meta.stmt_queue.push_back(format!("__AMBER_FUN_{name}{id}_v{variant}={result}"));
        format!("return 0")
    }
}