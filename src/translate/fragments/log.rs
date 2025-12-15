use super::fragment::{FragmentKind, FragmentRenderable};
use super::interpolable::InterpolableRenderType;
use crate::utils::TranslateMetadata;
use crate::modules::types::Type;

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

    fn should_use_echo(&self, fragment: &FragmentKind) -> bool {
        match fragment {
            FragmentKind::VarExpr(var) => {
                if matches!(var.kind, Type::Text) {
                    return false;
                }
                if let Type::Array(inner) = &var.kind {
                    if matches!(**inner, Type::Text) {
                        return false;
                    }
                }
                true
            }
            FragmentKind::Interpolable(interpolable) => {
                if interpolable.render_type != InterpolableRenderType::StringLiteral {
                    return false;
                }
                // Check first string chunk
                if let Some(first) = interpolable.strings.front() {
                     !first.is_empty() && !first.starts_with('-')
                } else {
                     false
                }
            }
            FragmentKind::List(list) => {
                if let Some(first) = list.values.first() {
                    self.should_use_echo(first)
                } else {
                    true
                }
            }
            FragmentKind::Raw(_) | FragmentKind::Arithmetic(_) => true,
            _ => false,
        }
    }
}


impl FragmentRenderable for LogFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        if self.should_use_echo(&self.value) {
            format!("echo {}", self.value.to_string(meta))
        } else {
            format!("printf '%s\\n' {}", self.value.to_string(meta))
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Log(self)
    }
}
