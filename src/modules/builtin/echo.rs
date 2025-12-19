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
        let value = self.value.translate(meta);
        // If the variable is an array, it's always passed as a variable expression
        let value = match value {
            FragmentKind::VarExpr(var) if var.kind.is_array() => {
                FragmentKind::VarExpr(var.with_array_to_string(true))
            }
            other => other,
        };

        FragmentKind::Log(LogFragment::new(value))
    }
}

impl DocumentationModule for Echo {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
