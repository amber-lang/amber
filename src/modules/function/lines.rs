use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct LinesInvocation {
    path: Box<Option<Expr>>,
}

impl Typed for LinesInvocation {
    fn get_type(&self) -> Type {
        Type::Array(Box::new(Type::Text))
    }
}

impl SyntaxModule<ParserMetadata> for LinesInvocation {
    syntax_name!("Lines Invocation");

    fn new() -> Self {
        LinesInvocation {
            path: Box::new(None)
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "lines")?;
        token(meta, "(")?;
        let tok = meta.get_current_token();
        let mut path = Expr::new();
        syntax(meta, &mut path)?;
        token(meta, ")")?;
        if path.get_type() != Type::Text {
            return error!(meta, tok, "Expected string");
        }
        self.path = Box::new(Some(path));
        Ok(())
    }
}

impl TranslateModule for LinesInvocation {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "".to_string()
    }
}

impl DocumentationModule for LinesInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
