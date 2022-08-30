use heraclitus_compiler::prelude::*;
use crate::{modules::{variable::get::VariableGet, expression::{expr::Expr, binop::expression_arms_of_type}, Type}, utils::ParserMetadata, translate::{module::TranslateModule, compute::{ArithOp, translate_computation}}};

#[derive(Debug)]
pub struct ShorthandAdd {
    var: VariableGet,
    expr: Box<Expr>,
    kind: Type
}

impl SyntaxModule<ParserMetadata> for ShorthandAdd {
    syntax_name!("Shorthand Add");

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
        token(meta, "+=")?;
        self.expr.parse(meta)?;
        let message = "Add operation can only add numbers or text";
        self.kind = expression_arms_of_type(meta, &self.var, &*self.expr, &[Type::Num, Type::Text], tok, message);
        Ok(())
    }
}

impl TranslateModule for ShorthandAdd {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> String {
        let var = self.var.translate(meta);
        let expr = self.expr.translate(meta);
        let name = self.var.name.clone();
        if self.kind == Type::Text {
            format!("{}+={}", name, expr)
        }
        else {
            format!("{}={}", name, translate_computation(meta, ArithOp::Add, Some(var), Some(expr)))
        }
    }
}