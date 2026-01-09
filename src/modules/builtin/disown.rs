use crate::fragments;
use crate::modules::prelude::*;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;

#[derive(Clone, Debug)]
pub struct Disown {}

impl SyntaxModule<ParserMetadata> for Disown {
    syntax_name!("disown");

    fn new() -> Self {
        Disown {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "disown")?;
        token(meta, "(")?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Disown {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Disown {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("disown")
    }
}

impl DocumentationModule for Disown {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}