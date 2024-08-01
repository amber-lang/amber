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
pub struct ShorthandDiv {
    var: String,
    expr: Box<Expr>,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool,
}

impl SyntaxModule<ParserMetadata> for ShorthandDiv {
    syntax_name!("Shorthand Div");

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
        token(meta, "/=")?;
        let variable = handle_variable_reference(meta, var_tok, &self.var)?;
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        syntax(meta, &mut *self.expr)?;
        if self.kind != self.expr.get_type() || !matches!(self.kind, Type::Num) {
            let message = format!(
                "Cannot divide variable of type '{}' by an expression of type '{}'",
                self.kind,
                self.expr.get_type()
            );
            let err = self.expr.get_error_message(meta).message(message);
            return Err(Failure::Loud(err));
        }
        Ok(())
    }
}

impl TranslateModule for ShorthandDiv {
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
            let eval = translate_computation_eval(meta, ArithOp::Div, Some(var), Some(expr));
            format!("eval \"{}={}\"", name, eval)
        } else {
            let eval = translate_computation(meta, ArithOp::Div, Some(var), Some(expr));
            format!("{}={}", name, eval)
        }
    }
}

impl DocumentationModule for ShorthandDiv {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
