use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{types::{Type, Typed}, expression::{expr::Expr, binop::{parse_left_expr, expression_arms_of_type}}}};
use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone)]
pub struct Range {
    from: Box<Expr>,
    to: Box<Expr>,
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
            to: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, self.from.as_mut(), "..")?;
        let tok = meta.get_current_token();
        token(meta, "..")?;
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
        format!("$(seq {} {})", self.from.translate(meta), self.to.translate(meta))
    }
}