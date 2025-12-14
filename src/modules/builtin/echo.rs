use crate::fragments;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
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
        token(meta, "echo")?;
        syntax(meta, &mut *self.value)?;
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
        let value_type = self.value.get_type();

        // Use echo for arrays (to preserve all elements on one line)
        if value_type.is_array() {
            return fragments!("echo ", self.value.translate(meta));
        }

        // Use echo for numeric types (numbers don't trigger echo flags like -e, -n)
        // Even negative numbers like -2, -123 are safe with echo
        if matches!(value_type, Type::Num | Type::Int) {
            return fragments!("echo ", self.value.translate(meta));
        }

        // Use echo for safe literals (pure text literals that don't start with dash)
        // Use printf for everything else (variables, interpolated strings, expressions, dash-prefixed literals)
        let is_safe_literal = match &self.value.value {
            Some(ExprType::Text(text)) => text.is_echo_safe_literal(),
            Some(ExprType::Number(_)) | Some(ExprType::Integer(_)) | Some(ExprType::Bool(_)) | Some(ExprType::Null(_)) => true,
            _ => false,
        };

        if is_safe_literal {
            fragments!("echo ", self.value.translate(meta))
        } else {
            fragments!("printf '%s\\n' ", self.value.translate(meta))
        }
    }
}

impl DocumentationModule for Echo {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
