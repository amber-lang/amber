use super::fragment::{FragmentKind, FragmentRenderable};
use super::interpolable::InterpolableRenderType;
use crate::modules::types::Type;
use crate::translate::fragments::interpolable::InterpolablePart;
use crate::utils::TranslateMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFragment {
    pub value: Box<FragmentKind>,
}

impl LogFragment {
    pub fn new(value: FragmentKind) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    fn should_use_printf(&self, fragment: &FragmentKind) -> bool {
        match fragment {
            FragmentKind::VarExpr(var) => {
                if matches!(var.kind, Type::Text) {
                    return true;
                }
                if let Type::Array(inner) = &var.kind {
                    if matches!(**inner, Type::Text) {
                        return true;
                    }
                }
                false
            }
            FragmentKind::Interpolable(interpolable) => {
                if interpolable.render_type != InterpolableRenderType::StringLiteral {
                    return true;
                }
                // `echo` prints backslashes literally like printf '%s', but
                // ShellCheck flags it (SC2028); use printf when any literal
                // part carries one.
                let has_backslash = interpolable.parts.iter().any(|p| {
                    matches!(p, InterpolablePart::String(s) if s.contains('\\'))
                });
                let front = interpolable.parts.front();
                match front {
                    Some(InterpolablePart::String(s)) => {
                        s.is_empty() || s.starts_with('-') || has_backslash
                    }
                    Some(InterpolablePart::Interp(_)) => true,
                    None => true,
                }
            }
            FragmentKind::List(list) => {
                if let Some(first) = list.values.first() {
                    self.should_use_printf(first)
                } else {
                    false
                }
            }
            FragmentKind::Raw(_) | FragmentKind::Arithmetic(_) => false,
            _ => true,
        }
    }
}

impl FragmentRenderable for LogFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        if self.should_use_printf(&self.value) {
            let value = self.value.to_string(meta);
            // An empty argument must stay explicit, otherwise printf gets a
            // format with no arguments (ShellCheck SC2183).
            let value = if value.is_empty() { "''".to_string() } else { value };
            format!("printf '%s\\n' {value}")
        } else {
            format!("echo {}", self.value.to_string(meta))
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Log(self)
    }
}
