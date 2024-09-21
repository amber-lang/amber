use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct GlobInvocation {
    args: Vec<Expr>,
}

impl Typed for GlobInvocation {
    fn get_type(&self) -> Type {
        Type::Array(Box::new(Type::Text))
    }
}

impl SyntaxModule<ParserMetadata> for GlobInvocation {
    syntax_name!("Glob Invocation");

    fn new() -> Self {
        GlobInvocation {
            args: vec![],
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "glob")?;
        token(meta, "(")?;
        loop {
            let tok = meta.get_current_token();
            let mut arg = Expr::new();
            syntax(meta, &mut arg)?;
            if arg.get_type() != Type::Text {
                return error!(meta, tok, "Expected string");
            }
            self.args.push(arg);
            match token(meta, ")") {
                Ok(_) => break,
                Err(_) => token(meta, ",")?,
            };
        }
        Ok(())
    }
}

impl TranslateModule for GlobInvocation {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        todo!()
    }
}

impl DocumentationModule for GlobInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
