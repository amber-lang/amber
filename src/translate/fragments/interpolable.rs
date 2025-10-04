use std::collections::VecDeque;

use crate::utils::TranslateMetadata;
use super::fragment::{FragmentKind, FragmentRenderable};

/// Represents a region that can be interpolated. Similarly to what Heraclitus returns when parsing a region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolableRenderType {
    /// This should be rendered to Bash's double quoted string
    StringLiteral,
    /// This should be rendered to Bash's global context expression (command)
    GlobalContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        self.balance_single_quotes();
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

    fn balance_single_quotes(&mut self) {
        let mut in_single_quotes = false;

        for s in &mut self.strings {
            // If previous chunk left us inside quotes, reopen at the start.
            if in_single_quotes {
                s.insert_str(0, "\"'");
            }

            let unescaped = count_unescaped_single_quotes(s);

            // If this chunk has an odd number of unescaped quotes, it toggles the region.
            if unescaped % 2 == 1 {
                in_single_quotes = !in_single_quotes;
                // Close the chunk locally so each piece is balanced.
                s.push_str("'\"");
            }
        }
    }

    fn translate_escaped_string(&self, string: String) -> String {
        let chars = string.chars();
        let mut result = String::new();
        for c in chars {
            match self.render_type {
                InterpolableRenderType::StringLiteral => {
                    match c {
                        '"' =>  result += r#"\""#,
                        '$' =>  result += r#"\$"#,
                        '`' =>  result += r#"\`"#,
                        '\\' =>  result += r#"\\"#,
                        '!' =>  result += r#""'!'""#,
                        _ => result.push(c),
                    }
                }
                InterpolableRenderType::GlobalContext => result.push(c),
            }
        }
        result
    }
}

/// Count single quotes that are NOT escaped by an odd number of preceding backslashes.
fn count_unescaped_single_quotes(s: &str) -> usize {
    let mut count = 0usize;
    let mut backslashes = 0usize;

    for b in s.bytes() {
        match b {
            b'\\' => backslashes += 1,
            b'\'' => {
                if backslashes % 2 == 0 {
                    count += 1;
                }
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
    }
    count
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_interpolable(render_type: InterpolableRenderType) -> InterpolableFragment {
        InterpolableFragment::new(vec![], vec![], render_type)
    }

    #[test]
    fn test_translate_escaped_string() {
        // Test StringLiteral translation
        let i_str = create_interpolable(InterpolableRenderType::StringLiteral);
        assert_eq!(i_str.translate_escaped_string(r#"hello"#.to_string()), r#"hello"#);
        assert_eq!(i_str.translate_escaped_string(r#"\"#.to_string()), r#"\\"#);
        assert_eq!(i_str.translate_escaped_string(r#"""#.to_string()), r#"\""#);
        assert_eq!(i_str.translate_escaped_string(r#"'"#.to_string()), r#"'"#);
        assert_eq!(i_str.translate_escaped_string(r#"$"#.to_string()), r#"\$"#);
        assert_eq!(i_str.translate_escaped_string(r#"\$"#.to_string()), r#"\\\$"#);
        assert_eq!(i_str.translate_escaped_string(r#"{"#.to_string()), r#"{"#);
        assert_eq!(i_str.translate_escaped_string(r#"`"#.to_string()), r#"\`"#);
        assert_eq!(i_str.translate_escaped_string(r#"!"#.to_string()), r#""'!'""#);
        assert_eq!(i_str.translate_escaped_string(r#"\ "#.to_string()), r#"\\ "#);
        assert_eq!(i_str.translate_escaped_string(r#"${var}"#.to_string()), r#"\${var}"#);

        // Test GlobalContext translation
        let i_glo = create_interpolable(InterpolableRenderType::GlobalContext);
        assert_eq!(i_glo.translate_escaped_string(r#"hello"#.to_string()), r#"hello"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\a"#.to_string()), r#"\a"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\"#.to_string()), r#"\"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\\"#.to_string()), r#"\\"#);
        assert_eq!(i_glo.translate_escaped_string(r#"""#.to_string()), r#"""#);
        assert_eq!(i_glo.translate_escaped_string(r#"'"#.to_string()), r#"'"#);
        assert_eq!(i_glo.translate_escaped_string(r#"$"#.to_string()), r#"$"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\$"#.to_string()), r#"\$"#);
        assert_eq!(i_glo.translate_escaped_string(r#"{"#.to_string()), r#"{"#);
        assert_eq!(i_glo.translate_escaped_string(r#"!"#.to_string()), r#"!"#);
        assert_eq!(i_glo.translate_escaped_string(r#"basename `pwd`"#.to_string()), r#"basename `pwd`"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\ "#.to_string()), r#"\ "#);
    }

    #[test]
    fn test_count_unescaped_single_quotes() {
        assert_eq!(count_unescaped_single_quotes(r#"foo"#), 0);
        assert_eq!(count_unescaped_single_quotes(r#"foo\'bar"#), 0);
        assert_eq!(count_unescaped_single_quotes(r#"foo'bar"#), 1);
        // even number of backslashes before quote -> not escaped
        assert_eq!(count_unescaped_single_quotes(r#"foo\\\\'bar"#), 1);
        assert_eq!(count_unescaped_single_quotes(r#"'\"'"#), 2);
        assert_eq!(count_unescaped_single_quotes(r#"'''"#), 3);
    }
}
