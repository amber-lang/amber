use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::{fragments, raw_fragment};
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::FragmentKind;
use crate::modules::types::{Type, Typed};
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone)]
pub struct Fail {
    pub expr: Expr,
    pub code: String,
    pub is_main: bool
}

impl Typed for Fail {
    fn get_type(&self) -> Type {
        self.expr.get_type()
    }
}

impl SyntaxModule<ParserMetadata> for Fail {
    syntax_name!("Fail");

    fn new() -> Self {
        Fail {
            expr: Expr::new(),
            code: String::new(),
            is_main: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "fail")?;
        let tok = meta.get_current_token();
        if !meta.context.is_fun_ctx && !meta.context.is_main_ctx {
            return error!(meta, tok => {
                message: "Fail statement outside of function or main",
                comment: "Fail statements can only be used inside of functions or the main block"
            });
        }
        self.is_main = meta.context.is_main_ctx;
        match integer(meta, vec![]) {
            Ok(value) => {
                if value == "0" {
                    return error!(meta, tok => {
                        message: "Invalid exit code",
                        comment: "Fail status must be a non-zero integer"
                    });
                }
                self.code = value;
            },
            Err(_) => {
                match syntax(meta, &mut self.expr) {
                    Ok(_) => {
                        if self.expr.get_type() != Type::Num {
                            return error!(meta, tok => {
                                message: "Invalid exit code",
                                comment: "Fail status must be a non-zero integer"
                            });
                        }
                    },
                    Err(_) => {
                        self.code = "1".to_string();
                    }
                }
            }
        }
        Ok(())
    }
}

impl TranslateModule for Fail {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let translate = if self.code.is_empty() {
            self.expr.translate(meta)
        } else {
            raw_fragment!("{}", &self.code)
        };
        if self.is_main {
            fragments!("exit ", translate)
        } else {
            // Clean the return value if the function fails
            let fun_meta = meta.fun_meta.as_ref().expect("Function name and return type not set");
            let stmt = VarStmtFragment::new(&fun_meta.mangled_name(), fun_meta.get_type(), fun_meta.default_return())
                .with_optimization_when_unused(false);
            meta.stmt_queue.push_back(stmt.to_frag());
            fragments!("return ", translate)
        }
    }
}

impl DocumentationModule for Fail {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
