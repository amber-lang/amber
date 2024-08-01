use super::BinOp;
use crate::docs::module::DocumentationModule;
use crate::modules::{
    expression::expr::Expr,
    types::{Type, Typed},
};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::ParserMetadata;
use crate::utils::TranslateMetadata;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Range {
    pub from: Box<Expr>,
    pub to: Box<Expr>,
    pub neq: bool,
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
            neq: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if self.from.get_type() != Type::Num {
            let l_type = self.from.get_type();
            let err = self.from.get_error_message(meta).message(format!(
                "Range can only work on type 'Num', but '{l_type}' was provided"
            ));
            return Err(Failure::Loud(err));
        }
        if self.to.get_type() != Type::Num {
            let r_type = self.to.get_type();
            let err = self.to.get_error_message(meta).message(format!(
                "Range can only work on type 'Num', but '{r_type}' was provided"
            ));
            return Err(Failure::Loud(err));
        }
        Ok(())
    }
}

impl TranslateModule for Range {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let from = self.from.translate(meta);
        let to = self.to.translate(meta);
        if self.neq {
            let to_neq = translate_computation(meta, ArithOp::Sub, Some(to), Some("1".to_string()));
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
