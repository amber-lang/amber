use heraclitus_compiler::prelude::*;
use crate::{modules::{variable::get::VariableGet, expression::{expr::Expr, binop::expression_arms_of_type}, Type}, utils::ParserMetadata, translate::{module::TranslateModule, compute::{ArithOp, translate_computation}}};

#[derive(Debug)]
pub struct ShorthandDiv {
    var: VariableGet,
    expr: Box<Expr>,
    kind: Type
}

impl SyntaxModule<ParserMetadata> for ShorthandDiv {
    syntax_name!("Shorthand Div");

    fn new() -> Self {
        Self {
            var: VariableGet::new(),
            expr: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.var.parse(meta)?;
        let tok = meta.get_current_token();
        token(meta, "/=")?;
        self.expr.parse(meta)?;
        let message = "Division operation can only divide numbers";
        self.kind = expression_arms_of_type(meta, &self.var, &*self.expr, &[Type::Num], tok, message);
        Ok(())
    }
}

impl TranslateModule for ShorthandDiv {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        let var = self.var.translate(meta);
        let expr = self.expr.translate(meta);
        let name = self.var.name.clone();
        format!("{}={}", name, translate_computation(meta, ArithOp::Div, Some(var), Some(expr)))
    }
}