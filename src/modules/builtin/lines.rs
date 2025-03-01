use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct LinesInvocation {
    path: Box<Option<Expr>>,
}

impl Typed for LinesInvocation {
    fn get_type(&self) -> Type {
        Type::array_of(Type::Text)
    }
}

impl SyntaxModule<ParserMetadata> for LinesInvocation {
    syntax_name!("Lines Invocation");

    fn new() -> Self {
        LinesInvocation {
            path: Box::new(None),
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
            let msg = format!(
                "Expected value of type 'Text' but got '{}'",
                path.get_type()
            );
            return error!(meta, tok, msg);
        }
        self.path = Box::new(Some(path));
        Ok(())
    }
}

impl TranslateModule for LinesInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        let temp = format!("__AMBER_LINE_{}", meta.gen_value_id());
        let path = (*self.path)
            .as_ref()
            .map(|p| p.translate(meta))
            .expect("Cannot read lines without provided path");
        let indent = TranslateMetadata::single_indent();
        let id = meta.gen_value_id();
        let value = meta.push_stmt_variable_lazy("__array", Some(id), Type::array_of(Type::Text), TranslationFragment::Empty);
        meta.stmt_queue.extend([
            fragments!(raw: "while IFS= read -r {temp}; do"),
            fragments!(raw: "{indent}{}+=(\"${}\")", value.get_name(), temp),
            fragments!("done <", path),
        ]);
        value.to_frag()
    }
}

impl LinesInvocation {
    pub fn translate_path(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        let path = (*self.path)
            .as_ref()
            .map(|p| p.translate(meta))
            .expect("Cannot read lines without provided path in iterator loop");
        path
    }
}

impl DocumentationModule for LinesInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
