use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Sleep {
    value: Expr,
}

impl SyntaxModule<ParserMetadata> for Sleep {
    syntax_name!("Sleep");

    fn new() -> Self {
        Sleep { value: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "sleep")?;
        token(meta, "(")?;
        syntax(meta, &mut self.value)?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Sleep {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value.typecheck(meta)?;
        let time_type = self.value.get_type();
        if time_type != Type::Int && time_type != Type::Num {
            let position = self.value.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `sleep` can only be used with values of type Int or Num",
                comment: format!("Given type: {}, expected type: {} or {}", time_type, Type::Int, Type::Num)
            });
        }
        Ok(())
    }
}

impl TranslateModule for Sleep {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("sleep ", self.value.translate(meta))
    }
}

impl DocumentationModule for Sleep {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
