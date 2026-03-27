use crate::docs::module::DocumentationModule;
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Shellversion {}

impl Typed for Shellversion {
    fn get_type(&self) -> Type {
        Type::array_of(Type::Int)
    }
}

impl SyntaxModule<ParserMetadata> for Shellversion {
    syntax_name!("Shellversion");

    fn new() -> Self {
        Shellversion {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "shellversion")?;
        token(meta, "(")?;
        token(meta, ")")?;
        meta.shellversion_used = true;
        Ok(())
    }
}

impl TypeCheckModule for Shellversion {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Shellversion {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new("EXEC_SHELL_VERSION", Type::array_of(Type::Int)).to_frag()
    }
}

impl DocumentationModule for Shellversion {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
