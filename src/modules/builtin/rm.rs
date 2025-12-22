use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;

#[derive(Debug, Clone)]
pub struct Rm {
    value: Expr,
}

impl SyntaxModule<ParserMetadata> for Rm {
    syntax_name!("Remove");

    fn new() -> Self {
        Rm { value: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "rm")?;
        token(meta, "(")?;
        syntax(meta, &mut self.value)?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Rm {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value.typecheck(meta)?;
        if self.value.get_type() != Type::Text {
            let position = self.value.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `rm` can only be used with 1st argument of type Text",
                comment: format!("Given type: {}, expected type: {}", self.value.get_type(), Type::Text)
            });
        }
        Ok(())
    }
}

impl TranslateModule for Rm {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("rm -f ", self.value.translate(meta))
    }
}

impl DocumentationModule for Rm {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
