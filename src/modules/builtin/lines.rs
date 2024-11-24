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
            let msg = format!("Expected value of type 'Text' but got '{}'", path.get_type());
            return error!(meta, tok, msg);
        }
        self.path = Box::new(Some(path));
        Ok(())
    }
}

impl TranslateModule for LinesInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = format!("__AMBER_ARRAY_{}", meta.gen_value_id());
        let temp = format!("__AMBER_LINE_{}", meta.gen_value_id());
        let path = (*self.path).as_ref()
            .map(|p| p.translate_eval(meta, false))
            .unwrap_or_default();
        let quote = meta.gen_quote();
        let dollar = meta.gen_dollar();
        let indent = TranslateMetadata::single_indent();
        let block = [
            format!("{name}=()"),
            format!("while IFS= read -r {temp}; do"),
            format!("{indent}{name}+=(\"${temp}\")"),
            format!("done <{path}"),
        ].join("\n");
        meta.stmt_queue.push_back(block);
        format!("{quote}{dollar}{{{name}[@]}}{quote}")
    }
}

impl LinesInvocation {
    pub fn surround_iter(&self, meta: &mut TranslateMetadata, name: &str) -> (String, String) {
        let path = (*self.path).as_ref()
            .map(|p| p.translate(meta))
            .unwrap_or_default();
        let prefix = format!("while IFS= read -r {name}; do");
        let suffix = format!("done <{path}");
        (prefix, suffix)
    }
}

impl DocumentationModule for LinesInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
