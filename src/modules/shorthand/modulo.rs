use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::{expr::Expr, binop::expression_arms_of_type};
use crate::modules::variable::{variable_name_extensions, handle_variable_reference};
use crate::translate::compute::translate_computation_eval;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::{module::TranslateModule, compute::{ArithOp, translate_computation}};
use crate::modules::types::{Type, Typed};

#[derive(Debug, Clone)]
pub struct ShorthandModulo {
    var: String,
    expr: Box<Expr>,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool
}

impl SyntaxModule<ParserMetadata> for ShorthandModulo {
    syntax_name!("Shorthand Modulo");

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
        let tok = meta.get_current_token();
        token(meta, "%=")?;
        let variable = handle_variable_reference(meta, var_tok, &self.var)?;
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        self.expr.parse(meta)?;
        let message = "Modulo operation can only be applied to numbers";
        let predicate = |kind| matches!(kind, Type::Num);
        expression_arms_of_type(meta, &self.kind, &self.expr.get_type(), predicate, tok, message)?;
        Ok(())
    }
}

impl TranslateModule for ShorthandModulo {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let expr = self.is_ref
            .then(|| self.expr.translate_eval(meta, true))
            .unwrap_or_else(|| self.expr.translate(meta));
        let name = match self.global_id {
            Some(id) => format!("__{id}_{}", self.var),
            None => if self.is_ref { format!("${{{}}}", self.var) } else { self.var.clone() }
        };
        let var = if self.is_ref { format!("\\${{{name}}}") } else { format!("${{{name}}}") };
        if self.is_ref {
            let expr = translate_computation_eval(meta, ArithOp::Modulo, Some(var), Some(expr));
            format!("eval \"{}={}\"", name, expr)
        } else {
            let expr = translate_computation(meta, ArithOp::Modulo, Some(var), Some(expr));
            format!("{}={}", name, expr)
        }
    }
}

impl DocumentationModule for ShorthandModulo {
    fn document(&self) -> String {
        "".to_string()
    }
}
