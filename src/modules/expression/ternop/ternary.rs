use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::binop::get_binop_position_info;
use crate::modules::types::{Type, Typed};
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Ternary {
    pub cond: Box<AlreadyParsedExpr>,
    pub true_expr: Box<AlreadyParsedExpr>,
    pub false_expr: Box<AlreadyParsedExpr>
}

impl Typed for Ternary {
    fn get_type(&self) -> Type {
        self.true_expr.get_type()
    }
}

impl SyntaxModule<ParserMetadata> for Ternary {
    syntax_name!("Ternary Expression");

    fn new() -> Self {
        Ternary {
            cond: Box::new(AlreadyParsedExpr::new()),
            true_expr: Box::new(AlreadyParsedExpr::new()),
            false_expr: Box::new(AlreadyParsedExpr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if self.cond.get_type() != Type::Bool {
            let msg = self.cond.get_error_message(meta)
                .message("Expected boolean expression in ternary condition");
            return Err(Failure::Loud(msg));
        }
        if self.true_expr.get_type() != self.false_expr.get_type() {
            let pos = get_binop_position_info(meta, &self.true_expr, &self.false_expr);
            let msg = Message::new_err_at_position(meta, pos)
                .message("Ternary operation can only be used on arguments of the same type")
                .comment(format!("Provided branches of type '{}' and '{}'.",
                    self.true_expr.get_type(),
                    self.false_expr.get_type()));
            return Err(Failure::Loud(msg));
        }
        Ok(())
    }
}

impl TranslateModule for Ternary {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let cond = self.cond.translate(meta);
        let true_expr = self.true_expr.translate(meta);
        let false_expr = self.false_expr.translate(meta);
        meta.gen_subprocess(&format!("if [ {} != 0 ]; then echo {}; else echo {}; fi", cond, true_expr, false_expr))
    }
}

impl DocumentationModule for Ternary {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
