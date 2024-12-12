use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::error_type_match;
use crate::modules::expression::expr::Expr;
use crate::modules::variable::{handle_variable_reference, prevent_constant_mutation, variable_name_extensions};
use crate::translate::compute::translate_computation_eval;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::{module::TranslateModule, compute::{ArithOp, translate_computation}};
use crate::modules::types::{Type, Typed};

#[derive(Debug, Clone)]
pub struct ShorthandAdd {
    var: String,
    expr: Box<Expr>,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool
}

impl SyntaxModule<ParserMetadata> for ShorthandAdd {
    syntax_name!("Shorthand Add");

    fn new() -> Self {
        Self {
            var: String::new(),
            expr: Box::new(Expr::new()),
            kind: Type::Null,
            global_id: None,
            is_ref: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let var_tok = meta.get_current_token();
        self.var = variable(meta, variable_name_extensions())?;
        token(meta, "+=")?;
        let variable = handle_variable_reference(meta, &var_tok, &self.var)?;
        prevent_constant_mutation(meta, &var_tok, &self.var, variable.is_const)?;
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        syntax(meta, &mut *self.expr)?;
        if self.kind != self.expr.get_type() || !matches!(self.kind, Type::Num | Type::Text | Type::Array(_)) {
            let msg = self.expr.get_error_message(meta);
            return error_type_match!(meta, msg, "add", self.expr, [Num, Text, Array]);
        }
        Ok(())
    }
}

impl TranslateModule for ShorthandAdd {
    //noinspection DuplicatedCode
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = if let Some(id) = self.global_id {
            format!("__{id}_{}", self.var)
        } else if self.is_ref {
            format!("${{{}}}", self.var)
        } else {
            self.var.clone()
        };
        match self.kind {
            Type::Text => {
                if self.is_ref {
                    let expr = self.expr.translate_eval(meta, true);
                    format!("eval \"{name}+={expr}\"")
                } else {
                    let expr = self.expr.translate(meta);
                    format!("{name}+={expr}")
                }
            }
            Type::Array(_) => {
                if self.is_ref {
                    let expr = self.expr.translate_eval(meta, true);
                    format!("eval \"{name}+=({expr})\"")
                } else {
                    let expr = self.expr.translate(meta);
                    format!("{name}+=({expr})")
                }
            }
            _ => {
                if self.is_ref {
                    let var = format!("\\${{{name}}}");
                    let expr = self.expr.translate_eval(meta, true);
                    let expr = translate_computation_eval(meta, ArithOp::Add, Some(var), Some(expr));
                    format!("eval \"{name}={expr}\"")
                } else {
                    let var = format!("${{{name}}}");
                    let expr = self.expr.translate(meta);
                    let expr = translate_computation(meta, ArithOp::Add, Some(var), Some(expr));
                    format!("{name}={expr}")
                }
            }
        }
    }
}

impl DocumentationModule for ShorthandAdd {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
