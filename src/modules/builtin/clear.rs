use crate::fragments;
use crate::modules::prelude::*;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;

#[derive(Debug, Clone)]
pub struct Clear {}

impl SyntaxModule<ParserMetadata> for Clear {
    syntax_name!("Clear");

    fn new() -> Self {
        Clear {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "clear")?;
        token(meta, "(")?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Clear {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Clear {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("clear")
    }
}

impl DocumentationModule for Clear {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
