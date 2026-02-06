use crate::fragments;
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Pid {}

impl Typed for Pid {
    fn get_type(&self) -> Type {
        Type::Int
    }
}

impl SyntaxModule<ParserMetadata> for Pid {
    syntax_name!("PidOfLastBackgroundCommand");

    fn new() -> Self {
        Pid {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "pid")?;
        token(meta, "(")?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Pid {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Pid {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let id = meta.gen_value_id();
        let var_stmt =
            VarStmtFragment::new("__pid", Type::Int, fragments!("$!")).with_global_id(id);
        meta.push_ephemeral_variable(var_stmt).to_frag()
    }
}

impl DocumentationModule for Pid {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
