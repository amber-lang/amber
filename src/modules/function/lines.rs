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

    fn append_let(&self, meta: &mut TranslateMetadata, name: &str, is_ref: bool) -> Option<String> {
        let id = meta.gen_value_id();
        let dollar = meta.gen_dollar();
        let path = (*self.path).as_ref().map(|p| p.translate(meta)).unwrap_or_default();
        let prefix = format!("while IFS= read -r __AMBER_LINE_{id}; do");
        let assign = if is_ref {
            format!("eval \"{dollar}{{{name}}}+=(\\\"{dollar}__AMBER_LINE_{id}\\\")\"")
        } else {
            format!("{name}+=(\"{dollar}__AMBER_LINE_{id}\")")
        };
        let suffix = format!("done <{path}");
        let append = [prefix, assign, suffix].join("\n");
        Some(append)
    }

    fn surround_iter(&self, meta: &mut TranslateMetadata, name: &str) -> Option<(String, String)> {
        let path = (*self.path).as_ref().map(|p| p.translate(meta)).unwrap_or_default();
        let prefix = format!("while IFS= read -r {name}; do");
        let suffix = format!("done <{path}");
        Some((prefix, suffix))
    }
}

impl DocumentationModule for LinesInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
