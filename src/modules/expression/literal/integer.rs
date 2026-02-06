use crate::docs::module::DocumentationModule;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Integer {
    pub value: String,
}

impl Typed for Integer {
    fn get_type(&self) -> Type {
        Type::Int
    }
}

impl SyntaxModule<ParserMetadata> for Integer {
    syntax_name!("Integer");

    fn new() -> Self {
        Integer {
            value: String::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Ok(sym) = token(meta, "-") {
            self.value.push_str(&sym);
        }
        let int = integer(meta, vec![])?;
        self.value.push_str(&int);
        Ok(())
    }
}

impl TypeCheckModule for Integer {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Integer {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        RawFragment::from(self.value.to_string()).to_frag()
    }
}

impl DocumentationModule for Integer {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
