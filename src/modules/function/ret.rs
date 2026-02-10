use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::function_metadata::FunctionMetadata;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Return {
    pub expr: Expr,
}

impl Typed for Return {
    fn get_type(&self) -> Type {
        self.expr.get_type()
    }
}

impl SyntaxModule<ParserMetadata> for Return {
    syntax_name!("Return");

    fn new() -> Self {
        Return { expr: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "return")?;
        if !meta.context.is_fun_ctx {
            let tok = meta.get_current_token();
            return error!(meta, tok => {
                message: "Return statement outside of function",
                comment: "Return statements can only be used inside of functions"
            });
        }
        syntax(meta, &mut self.expr)?;
        Ok(())
    }
}

impl TypeCheckModule for Return {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;

        let ret_type = meta.context.fun_ret_type.as_ref();
        let expr_type = &self.expr.get_type();
        match ret_type {
            Some(ret_type) => {
                if !expr_type.is_allowed_in(ret_type) {
                    let tok = meta.get_current_token();
                    return error!(meta, tok => {
                        message: "Return type does not match function return type",
                        comment: format!("Given type: {}, expected type: {}", expr_type, ret_type)
                    });
                }
            }
            None => {
                meta.context.fun_ret_type = Some(expr_type.clone());
            }
        }
        Ok(())
    }
}

impl TranslateModule for Return {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let fun_name = meta
            .fun_meta
            .as_ref()
            .map(FunctionMetadata::mangled_name)
            .expect("Function name and return type not set");
        let result = self.expr.translate(meta);
        let var_stmt = VarStmtFragment::new(&fun_name, self.expr.get_type(), result)
            .with_optimization_when_unused(false);
        meta.stmt_queue.push_back(var_stmt.to_frag());
        fragments!("return 0")
    }
}

impl DocumentationModule for Return {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
