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
        let fraction = meta.get_current_token();
        self.value.push_str(&integer(meta, vec![])?);
        // A second decimal point (e.g. `123.45.67`) makes the literal malformed
        if let Some(dot) = Self::extra_decimal_point(meta, fraction) {
            return error!(meta, Some(dot), "Invalid number literal");
        }
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
    /// Returns the token of a second decimal point that directly follows
    /// the fractional digits (`.67` in `123.45.67`)
    fn extra_decimal_point(meta: &ParserMetadata, fraction: Option<Token>) -> Option<Token> {
        let adjacent = |prev: &Token, index: usize| {
            meta.get_token_at(index)
                .filter(|token| token.start == prev.start + prev.word.chars().count())
        };
        let index = meta.get_index();
        let dot = adjacent(&fraction?, index).filter(|token| token.word == ".")?;
        let digits = adjacent(&dot, index + 1)?;
        digits
            .word
            .chars()
            .all(|ch| ch.is_ascii_digit())
            .then_some(dot)
    }

    pub fn get_integer_value(&self) -> Option<isize> {
        let value = self.value.parse().unwrap_or_default();
        Some(value)
    }
}

crate::impl_documentation_noop!(Number);
