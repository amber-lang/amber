use crate::fragments;
use crate::modules::expression::expr::Expr;
use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};

#[derive(Debug, Clone)]
pub struct Touch {
    value: Expr,
}

impl SyntaxModule<ParserMetadata> for Touch {
    syntax_name!("TouchFile");

    fn new() -> Self {
        Touch { value: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "touch")?;
        syntax(meta, &mut self.value)?;
        Ok(())
    }
}

impl TypeCheckModule for Touch {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // First, type-check the nested expression
        self.value.typecheck(meta)?;

        // Then check if it's the correct type
        let path_type = self.value.get_type();
        if path_type != Type::Text {
            let position = self.value.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `touch` can only be used with values of type Text",
                comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
            });
        }
        Ok(())
    }
}

impl TranslateModule for Touch {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("touch ", self.value.translate(meta))
    }
}

impl DocumentationModule for Touch {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
