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
        Type::Int
    }
}

impl SyntaxModule<ParserMetadata> for Status {
    syntax_name!("Status");

    fn new() -> Self {
        Status
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let position = meta.get_index();

        token(meta, "status")?;
        if token(meta, "(").is_ok() {
            token(meta, ")")?;
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling status without parentheses is deprecated");
            meta.add_message(warning);
        }
        Ok(())
    }
}

impl TypeCheckModule for Status {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Status {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new("__status", Type::Int).to_frag()
    }
}

impl DocumentationModule for Status {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
