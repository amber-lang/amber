use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "shellname"]
#[kind = "builtin_expr"]
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
        Ok(())
    }
}

impl TypeCheckModule for Shellname {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Shellname {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Only set shellname_used flag for standalone usage
        // (not when it's being folded via try_fold_comparison)
        meta.shellname_used = true;
        VarExprFragment::new("EXEC_SHELL", Type::Text).to_frag()
    }
}

crate::impl_documentation_noop!(Shellname);
