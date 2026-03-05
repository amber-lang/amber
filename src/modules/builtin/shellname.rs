use crate::docs::module::DocumentationModule;
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Shellname {}

impl Typed for Shellname {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Shellname {
    syntax_name!("Shellname");

    fn new() -> Self {
        Shellname {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "shellname")?;
        token(meta, "(")?;
        token(meta, ")")?;
        meta.shellname_used = true;
        Ok(())
    }
}

impl TypeCheckModule for Shellname {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Shellname {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new("EXEC_SHELL", Type::Text).to_frag()
    }
}

impl DocumentationModule for Shellname {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
