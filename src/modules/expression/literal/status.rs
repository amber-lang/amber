use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::prelude::FragmentKind;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone)]
pub struct Status;

impl Typed for Status {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Status {
    syntax_name!("Status");

    fn new() -> Self {
        Status
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "status")?;
        Ok(())
    }
}

impl TranslateModule for Status {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new("__status", Type::Num).to_frag()
    }
}

impl DocumentationModule for Status {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
