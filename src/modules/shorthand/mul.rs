use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{handle_variable_reference, variable_name_extensions};
use crate::translate::compute::translate_computation_eval;
use crate::translate::{
    compute::{translate_computation, ArithOp},
    module::TranslateModule,
};
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct ShorthandMul {
    var: String,
    expr: Box<Expr>,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool,
}

impl SyntaxModule<ParserMetadata> for ShorthandMul {
    syntax_name!("Shorthand Mul");

    fn new() -> Self {
        Self {
            var: String::new(),
            expr: Box::new(Expr::new()),
            kind: Type::Null,
            global_id: None,
            is_ref: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let var_tok = meta.get_current_token();
        self.var = variable(meta, variable_name_extensions())?;
        token(meta, "*=")?;
        let variable = handle_variable_reference(meta, var_tok, &self.var)?;
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        syntax(meta, &mut *self.expr)?;
        if self.kind != self.expr.get_type() || !matches!(self.kind, Type::Num) {
            let message = format!(
                "Cannot multiply an expression of type '{}' with a variable of type '{}'",
                self.expr.get_type(),
                self.kind
            );
            let err = self.expr.get_error_message(meta).message(message);
            return Err(Failure::Loud(err));
        }
        Ok(())
    }
}

impl TranslateModule for ShorthandMul {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let expr = self
            .is_ref
            .then(|| self.expr.translate_eval(meta, true))
            .unwrap_or_else(|| self.expr.translate(meta));
        let name = match self.global_id {
            Some(id) => format!("__{id}_{}", self.var),
            None => {
                if self.is_ref {
                    format!("${{{}}}", self.var)
                } else {
                    self.var.clone()
                }
            }
        };
        let var = if self.is_ref {
            format!("\\${{{name}}}")
        } else {
            format!("${{{name}}}")
        };
        if self.is_ref {
            let expr = translate_computation_eval(meta, ArithOp::Mul, Some(var), Some(expr));
            format!("eval \"{}={}\"", name, expr)
        } else {
            let expr = translate_computation(meta, ArithOp::Mul, Some(var), Some(expr));
            format!("{}={}", name, expr)
        }
    }
}

impl DocumentationModule for ShorthandMul {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
