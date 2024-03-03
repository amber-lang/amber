use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::binop::parse_left_expr;
use crate::modules::expression::expr::Expr;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type, parse_type};

#[derive(Debug, Clone)]
pub struct Is {
    expr: Box<Expr>,
    kind: Type
}

impl Typed for Is {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl SyntaxModule<ParserMetadata> for Is {
    syntax_name!("Add");

    fn new() -> Self {
        Is {
            expr: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.expr, "is")?;
        token(meta, "is")?;
        self.kind = parse_type(meta)?;
        Ok(())
    }
}

impl TranslateModule for Is {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        if self.expr.get_type() == self.kind {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }
}

impl DocumentationModule for Is {
    fn document(&self) -> String {
        "".to_string()
    }
}
