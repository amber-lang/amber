use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::error_type_match;
use crate::modules::expression::expr::Expr;
use crate::modules::variable::{handle_variable_reference, prevent_constant_mutation, variable_name_extensions};
use crate::translate::compute::translate_computation_eval;
use crate::translate::compute::ArithOp;
use crate::modules::types::{Type, Typed};

#[derive(Debug, Clone)]
pub struct ShorthandSub {
    var: String,
    expr: Box<Expr>,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool
}

impl SyntaxModule<ParserMetadata> for ShorthandSub {
    syntax_name!("Shorthand Sub");

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
        token(meta, "-=")?;
        let variable = handle_variable_reference(meta, &var_tok, &self.var)?;
        prevent_constant_mutation(meta, &var_tok, &self.var, variable.is_const)?;
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        syntax(meta, &mut *self.expr)?;
        if self.kind != self.expr.get_type() || !matches!(self.kind, Type::Num) {
            let msg = self.expr.get_error_message(meta);
            return error_type_match!(meta, msg, "subtract", self.expr, [Num, Text, Array]);
        }
        Ok(())
    }
}

impl TranslateModule for ShorthandSub {
    //noinspection DuplicatedCode
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        let var = VarFragment::new(&self.var, self.kind.clone(), self.is_ref, self.global_id);
        let expr = self.expr.translate_eval(meta, self.is_ref);
        let expr = translate_computation_eval(meta, ArithOp::Sub, Some(var.to_frag()), Some(expr), self.is_ref);
        let (stmt, _var) = meta.gen_stmt_variable(&self.var, self.global_id, self.kind.clone(), self.is_ref, None, "=", expr);
        stmt
    }
}

impl DocumentationModule for ShorthandSub {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
