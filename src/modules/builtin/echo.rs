use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Echo {
    value: Box<Expr>,
}

impl SyntaxModule<ParserMetadata> for Echo {
    syntax_name!("Log");

    fn new() -> Self {
        Echo {
            value: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let position = meta.get_index();
        token(meta, "echo")?;

        if token(meta, "(").is_ok() {
            syntax(meta, &mut *self.value)?;
            token(meta, ")")?;
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling a builtin without parentheses is deprecated");
            meta.add_message(warning);
            syntax(meta, &mut *self.value)?;
        }
        Ok(())
    }
}

impl TypeCheckModule for Echo {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value.typecheck(meta)
    }
}

impl TranslateModule for Echo {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("echo ", self.value.translate(meta))
    }
}

impl DocumentationModule for Echo {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
