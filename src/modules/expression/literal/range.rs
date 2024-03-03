use heraclitus_compiler::prelude::*;
use crate::{docs::module::DocumentationModule, modules::{expression::{binop::{expression_arms_of_type, parse_left_expr}, expr::Expr}, types::{Type, Typed}}, translate::compute::{translate_computation, ArithOp}, utils::metadata::ParserMetadata};
use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;

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
        parse_left_expr(meta, self.from.as_mut(), "..")?;
        let tok = meta.get_current_token();
        token(meta, "..")?;
        token(meta, "=").is_err().then(|| self.neq = true);
        syntax(meta, self.to.as_mut())?;
        let l_type = self.from.get_type();
        let r_type = self.to.get_type();
        let message = format!("Cannot create a range starting from value of type '{l_type}' up until value of type '{r_type}'");
        let predicate = |kind| matches!(kind, Type::Num);
        expression_arms_of_type(meta, &l_type, &r_type, predicate, tok, &message)?;
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
    fn document(&self) -> String {
        "".to_string()
    }
}
