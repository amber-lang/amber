use heraclitus_compiler::prelude::*;
use crate::{modules::{variable::{variable_name_extensions, handle_variable_reference}, expression::{expr::Expr, binop::expression_arms_of_type}, Type}, utils::ParserMetadata, translate::{module::TranslateModule, compute::{ArithOp, translate_computation}}};
use crate::modules::Typed;

#[derive(Debug, Clone)]
pub struct ShorthandMul {
    var: String,
    expr: Box<Expr>,
    kind: Type
}

impl SyntaxModule<ParserMetadata> for ShorthandMul {
    syntax_name!("Shorthand Mul");

    fn new() -> Self {
        Self {
            var: String::new(),
            expr: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let var_tok = meta.get_current_token();
        self.var = variable(meta, variable_name_extensions())?;
        let tok = meta.get_current_token();
        token(meta, "*=")?;
        self.kind = handle_variable_reference(meta, var_tok, &self.var);
        self.expr.parse(meta)?;
        let message = "Multiplication operation can only multiply numbers";
        expression_arms_of_type(meta, &self.kind, &self.expr.get_type(), &[Type::Num, Type::Text], tok, message);
        Ok(())
    }
}

impl TranslateModule for ShorthandMul {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        let expr = self.expr.translate(meta);
        let name = self.var.clone();
        format!("{}={}", name, translate_computation(meta, ArithOp::Mul, Some(name.clone()), Some(expr)))
    }
}