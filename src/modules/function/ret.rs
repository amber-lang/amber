use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::utils::function_interface::FunctionInterface;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::block::Block;
use crate::modules::types::parse_type;

#[derive(Debug, Clone)]
pub struct Ret {
    pub expr: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for Ret {
    syntax_name!("Ret");

    fn new() -> Self {
        Ret {
            expr: Box::new(Expr::new())
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
        syntax(meta, &mut *self.expr)?;
        match meta.context.fun_ret_type {
            Some(ret_type) => if ret_type != self.expr.get_type() {
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
        // FIXME: Set return value to a variable and terminate the expression
        "return".to_string()
    }
}