use crate::modules::expression::interpolated_region::{InterpolatedRegionType, parse_interpolated_region};
use crate::modules::expression::literal::text::TextPart;

use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::fragments::interpolable::InterpolablePart;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "param"]
#[kind = "builtin_expr"]
pub struct Param {
    value: String,
    token: Option<Token>,
}

impl Typed for Param {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl Param {
    fn parse_name(&self, meta: &mut ParserMetadata) -> Result<String, Failure> {
        let mut r = String::new();
        for p in parse_interpolated_region(meta, &InterpolatedRegionType::Text)? {
            if let TextPart::String(s) = p {
                r += s.as_str();
            } else {
                let m = Message::new_err_at_token(meta, self.token.clone()).message("Only plain strings are allowed in param()");
                return Err(Failure::Loud(m))
            }
        }
        Ok(r)
    }
}

impl SyntaxModule<ParserMetadata> for Param {
    syntax_name!("Param");

    fn new() -> Self {
        Param {
            value: String::new(),
            token: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "param")?;
        self.token = meta.get_current_token();
        token(meta, "(")?;
        let n = self.parse_name(meta)?;
        let d = if token(meta, ",").is_ok() {
            Some(self.parse_name(meta)?)
        } else {
            None
        };
        self.value = match std::env::var(&n) {
            Ok(v) => v,
            Err(e) => match e {
                std::env::VarError::NotPresent => {
                    d.ok_or_else(|| {
                        Failure::Loud(Message::new_err_at_token(meta, self.token.clone()).message(format!("{}: no such environment variable", n)))
                    })?
                },
                std::env::VarError::NotUnicode(_) => {
                    return Err(Failure::Loud(Message::new_err_at_token(meta, self.token.clone()).message(format!("{}: unreadable environment variable", n))))
                }
            }
        };
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Param {
    fn typecheck(&mut self, _: &mut ParserMetadata) -> SyntaxResult {
        // Just plain Text
        Ok(())
    }
}

impl TranslateModule for Param {
    fn translate(&self, _: &mut TranslateMetadata) -> FragmentKind {
        InterpolableFragment::new(vec![InterpolablePart::String(self.value.clone())], InterpolableRenderType::StringLiteral).to_frag()
    }
}

crate::impl_documentation_noop!(Param);
