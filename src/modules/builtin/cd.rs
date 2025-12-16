use crate::fragments;
use crate::modules::expression::expr::Expr;
use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};

#[derive(Debug, Clone)]
pub struct Cd {
    value: Expr,
}

impl SyntaxModule<ParserMetadata> for Cd {
    syntax_name!("ChangeDirectory");

    fn new() -> Self {
        Cd { value: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let position = meta.get_index();
        token(meta, "cd")?;

        if token(meta, "(").is_ok() {
            syntax(meta, &mut self.value)?;
            token(meta, ")")?;
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling a builtin without parentheses is deprecated");
            meta.add_message(warning);
            syntax(meta, &mut self.value)?;
        }
        Ok(())
    }
}

impl TypeCheckModule for Cd {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // First, type-check the nested expression
        self.value.typecheck(meta)?;

        // Then check if it's the correct type
        let path_type = self.value.get_type();
        if path_type != Type::Text {
            let position = self.value.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `cd` can only be used with values of type Text",
                comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
            });
        }
        Ok(())
    }
}

impl TranslateModule for Cd {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("cd ", self.value.translate(meta), " || exit")
    }
}

impl DocumentationModule for Cd {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
