use std::collections::VecDeque;

use crate::utils::TranslateMetadata;
use super::fragment::{FragmentKind, FragmentRenderable};

/// Represents a region that can be interpolated. Similarily to what Heraclitus returns when parsing a region.
#[derive(Debug, Clone)]
pub enum InterpolableRenderType {
    /// This should be rendered to Bash's double quoted string
    StringLiteral,
    /// This should be rendered to Bash's global context expression (command)
    GlobalContext,
}

#[derive(Debug, Clone)]
pub struct InterpolableFragment {
    pub strings: VecDeque<String>,
    pub interps: VecDeque<FragmentKind>,
    pub render_type: InterpolableRenderType,
    pub quoted: bool,
}

impl InterpolableFragment {
    pub fn new(strings: Vec<String>, interps: Vec<FragmentKind>, render_type: InterpolableRenderType) -> Self {
        InterpolableFragment {
            strings: VecDeque::from_iter(strings),
            interps: VecDeque::from_iter(interps),
            render_type,
            quoted: true,
        }
    }

    pub fn with_render_type(mut self, render_type: InterpolableRenderType) -> Self {
        self.render_type = render_type;
        self
    }

    pub fn with_quotes(mut self, quoted: bool) -> Self {
        self.quoted = quoted;
        self
    }

    pub fn render_interpolated_region(mut self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        while let Some(string) = self.strings.pop_front() {
            result.push(self.translate_escaped_string(string));
            if let Some(translated) = self.interps.pop_front() {
                // Quotes inside of interpolable strings are not necessary
                if let FragmentKind::Interpolable(mut interpolable) = translated {
                    interpolable = interpolable.with_render_type(InterpolableRenderType::GlobalContext);
                    result.push(interpolable.to_string(meta));
                } else {
                    result.push(translated.to_string(meta));
                }
            }
        }
        result.join("")
    }

    fn translate_escaped_string(&self, string: String) -> String {
        let mut chars = string.chars().peekable();
        let mut result = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result += r#"\""#,
                    InterpolableRenderType::GlobalContext => result += r#"""#,
                }
                '$' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result += r"$",
                    InterpolableRenderType::GlobalContext => result += r"\$",
                }
                '`' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result += r"`",
                    InterpolableRenderType::GlobalContext => result += r"\`",
                }
                '!' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result += r#""'!'""#,
                    InterpolableRenderType::GlobalContext => result += r"!",
                }
                '\\' => {
                    // Escape symbols
                    match chars.peek() {
                        Some('\n') => {}
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('0') => result.push('\0'),
                        Some('{') => result.push('{'),
                        Some('$') => result.push('$'),
                        Some('\'') => match self.render_type {
                            InterpolableRenderType::StringLiteral => result += r#"'"#,
                            InterpolableRenderType::GlobalContext => result += r#"\'"#,
                        }
                        Some('"') => match self.render_type {
                            InterpolableRenderType::StringLiteral => result += r#"\""#,
                            InterpolableRenderType::GlobalContext => result += r#"""#,
                        }
                        Some('\\') => match self.render_type {
                            InterpolableRenderType::StringLiteral => result += r#"\\"#,
                            InterpolableRenderType::GlobalContext => result += r#"\"#,
                        }
                        _ => {
                            result.push(c);
                            continue;
                        }
                    }
                    chars.next();
                },
                _ => result.push(c)
            }
        }
        result
    }
}

impl FragmentRenderable for InterpolableFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let render_type = self.render_type.clone();
        let quote = if self.quoted { meta.gen_quote() } else { "" };
        let result = self.render_interpolated_region(meta);
        match render_type {
            InterpolableRenderType::StringLiteral => format!("{quote}{result}{quote}"),
            InterpolableRenderType::GlobalContext => result.trim().to_string(),
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Interpolable(self)
    }
}
