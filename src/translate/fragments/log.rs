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
                // Check first string chunk
                if let Some(first) = interpolable.strings.front() {
                     first.is_empty() || first.starts_with('-')
                } else {
                     true
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
            format!("printf '%s\\n' {}", self.value.to_string(meta))
        } else {
            format!("echo {}", self.value.to_string(meta))
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Log(self)
    }
}
