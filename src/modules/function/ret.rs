use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::fragments::return_variable_name;
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "return"]
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
        let fun = meta
            .fun_frag_sig
            .as_ref()
            .expect("Function name and return type not set");
        let fun_name = return_variable_name(&fun.name, fun.declaration_id, fun.variant_id);
        let result = self.expr.translate(meta);
        // Returning a call to the same function already writes the callee's
        // result into this function's return binding, so the re-assignment
        // would be a self-assignment (ShellCheck SC2269).
        let is_self_return = match &result {
            FragmentKind::VarExpr(var) => {
                let name = var.get_name();
                name == fun_name || name.starts_with(&format!("{fun_name}__"))
            }
            _ => false,
        };
        if is_self_return {
            // Returning a call to the same function already writes the
            // callee's result into this function's return binding: drop the
            // ephemeral callsite binding instead of re-assigning the variable
            // to itself (ShellCheck SC2269).
            if let Some(FragmentKind::VarStmt(stmt)) = meta.stmt_queue.pop_back() {
                if !(stmt.is_ephemeral && stmt.get_name().starts_with(&fun_name)) {
                    meta.stmt_queue.push_back(FragmentKind::VarStmt(stmt));
                }
            }
        } else {
            let var_stmt = VarStmtFragment::new(&fun_name, self.expr.get_type(), result)
                .with_optimization_when_unused(false);
            meta.stmt_queue.push_back(var_stmt.to_frag());
        }
        fragments!("return 0")
    }
}

crate::impl_documentation_noop!(Return);
