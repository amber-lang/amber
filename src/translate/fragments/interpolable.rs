use std::collections::VecDeque;

use crate::utils::TranslateMetadata;
use super::fragment::{TranslationFragment, TranslationFragmentable};

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
    pub interps: VecDeque<TranslationFragment>,
    pub render_type: InterpolableRenderType,
}

impl InterpolableFragment {
    pub fn new(strings: Vec<String>, interps: Vec<TranslationFragment>, render_type: InterpolableRenderType) -> Self {
        InterpolableFragment {
            strings: VecDeque::from_iter(strings),
            interps: VecDeque::from_iter(interps),
            render_type,
        }
    }

    pub fn set_render_type(mut self, render_type: InterpolableRenderType) -> Self {
        self.render_type = render_type;
        self
    }

    pub fn render_interpolated_region(mut self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        loop {
            match self.strings.pop_front() {
                Some(string) => {
                    result.push(self.translate_escaped_string(string));
                }
                None => break
            }
            match self.interps.pop_front() {
                Some(translated) => {
                    // Quotes inside of interpolable strings are not necessary
                    if let TranslationFragment::Interpolable(mut interpolable) = translated {
                        interpolable = interpolable.set_render_type(InterpolableRenderType::GlobalContext);
                        result.push(interpolable.render(meta));
                    } else {
                        result.push(translated.render(meta));
                    }
                }
                None => break
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
                    InterpolableRenderType::StringLiteral => result += "\\\"",
                    InterpolableRenderType::GlobalContext => result.push('"'),
                }
                '$' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result.push('$'),
                    InterpolableRenderType::GlobalContext => result += "\\$",
                }
                '`' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result.push('`'),
                    InterpolableRenderType::GlobalContext => result += "\\`",
                }
                '!' => match self.render_type {
                    InterpolableRenderType::StringLiteral => result += "\"'!'\"",
                    InterpolableRenderType::GlobalContext => result.push('!'),
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
                            InterpolableRenderType::StringLiteral => result.push('\''),
                            InterpolableRenderType::GlobalContext => result += "\\'",
                        }
                        Some('\"') => match self.render_type {
                            InterpolableRenderType::StringLiteral => result += "\\\"",
                            InterpolableRenderType::GlobalContext => result.push('"'),
                        }
                        Some('\\') => match self.render_type {
                            InterpolableRenderType::StringLiteral => result += "\\\\",
                            InterpolableRenderType::GlobalContext => result.push('\\'),
                        }
                        _ => result.push(c)
                    }
                    chars.next();
                },
                _ => result.push(c)
            }
        }
        result
    }
}

impl TranslationFragmentable for InterpolableFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        let render_type = self.render_type.clone();
        let result = self.render_interpolated_region(meta);
        let quote = meta.gen_quote();
        match render_type {
            InterpolableRenderType::StringLiteral => format!("{quote}{result}{quote}"),
            InterpolableRenderType::GlobalContext => result.trim().to_string(),
        }
    }

    fn to_frag(self) -> TranslationFragment {
        TranslationFragment::Interpolable(self)
    }
}
