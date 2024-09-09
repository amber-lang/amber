use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::ExprType;
use crate::{handle_binop, error_type_match};
use crate::modules::{expression::expr::Expr, types::{Type, Typed}};
use crate::utils::metadata::ParserMetadata;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Range {
    from: Box<Expr>,
    to: Box<Expr>,
    neq: bool
}

impl Typed for Range {
    fn get_type(&self) -> Type {
        Type::Array(Box::new(Type::Num))
    }
}

impl BinOp for Range {
    fn set_left(&mut self, left: Expr) {
        self.from = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.to = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "..")?;
        self.neq = token(meta, "=").is_err();
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Range {
    syntax_name!("Range");

    fn new() -> Self {
        Range {
            from: Box::new(Expr::new()),
            to: Box::new(Expr::new()),
            neq: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        handle_binop!(meta, "apply range operator for", self.from, self.to, [Num])?;
        Ok(())
    }
}

impl TranslateModule for Range {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let from = self.from.translate(meta);
        let to = self.to.translate(meta);
        if self.neq {
            let to_neq = match &self.to.value.as_ref().unwrap() {
                ExprType::Number(_) => (to.parse::<i64>().unwrap() - 1_i64).to_string(),
                ExprType::VariableGet(_) => translate_computation(meta, ArithOp::Sub, Some(to), Some("1".to_string())),
                _ => unreachable!("range expression must be either Number or VariableGet"),
            };
            meta.gen_subprocess(&format!("seq {} {}", from, to_neq))
        } else {
            meta.gen_subprocess(&format!("seq {} {}", from, to))
        }
    }
}

impl DocumentationModule for Range {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
