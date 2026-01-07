use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::raw_fragment;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Pwd {}

impl Typed for Pwd {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Pwd {
    syntax_name!("PrintWorkingDirectory");

    fn new() -> Self {
        Pwd {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "pwd")?;
        token(meta, "(")?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Pwd {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Pwd {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let id = meta.gen_value_id();
        let var_stmt =
            VarStmtFragment::new("__pwd", Type::Text, raw_fragment!("\"$PWD\"")).with_global_id(id);
        meta.push_ephemeral_variable(var_stmt).to_frag()
    }
}

impl DocumentationModule for Pwd {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
