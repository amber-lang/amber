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
        let ret_type = meta.context.fun_ret_type.as_ref();
        let expr_type = &self.expr.get_type();
        // Unpacking Failable types
        let (ret_type, expr_type) = match (ret_type, expr_type) {
                types @ (Some(Type::Failable(_)), Type::Failable(_)) => types,
                (Some(Type::Failable(ret_type)), expr_type) => (Some(ret_type.as_ref()), expr_type),
                (Some(ret_type), Type::Failable(expr_type)) => (Some(ret_type), expr_type.as_ref()),
                types => types
        };
        match ret_type {
            Some(ret_type) => {
                if ret_type != expr_type {
                    return error!(meta, tok => {
                        message: "Return type does not match function return type",
                        comment: format!("Given type: {}, expected type: {}", expr_type, ret_type)
                    });
                }
            },
            None => {
                meta.context.fun_ret_type = Some(expr_type.clone());
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
        meta.stmt_queue.push_back(format!("__AF_{name}{id}_v{variant}={result}"));
        "return 0".to_string()
    }
}

impl DocumentationModule for Return {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
