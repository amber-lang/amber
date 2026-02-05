use crate::docs::module::DocumentationModule;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Number {
    value: String,
}

impl Typed for Number {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl SyntaxModule<ParserMetadata> for Number {
    syntax_name!("Number");

    fn new() -> Self {
        Number {
            value: String::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Ok(sym) = token(meta, "-") {
            self.value.push_str(&sym);
        }
        if let Ok(value) = integer(meta, vec![]) {
            self.value.push_str(&value);
        }
        let sym = token(meta, ".")?;
        self.value.push_str(&sym);
        self.value.push_str(&integer(meta, vec![])?);
        Ok(())
    }
}

impl TypeCheckModule for Number {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Number {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        RawFragment::from(self.value.to_string()).to_frag()
    }
}

impl Number {
    pub fn get_integer_value(&self) -> Option<isize> {
        let value = self.value.parse().unwrap_or_default();
        Some(value)
    }
}

impl DocumentationModule for Number {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
