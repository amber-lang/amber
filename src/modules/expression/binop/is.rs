use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::modules::types::AlreadyParsedType;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Is {
    pub expr: Box<AlreadyParsedExpr>,
    pub kind: AlreadyParsedType
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
            expr: Box::new(AlreadyParsedExpr::new()),
            kind: AlreadyParsedType::Null
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
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
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
