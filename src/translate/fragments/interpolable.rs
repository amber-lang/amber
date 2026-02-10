use std::collections::VecDeque;

use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

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
    pub fn new(
        strings: Vec<String>,
        interps: Vec<FragmentKind>,
        render_type: InterpolableRenderType,
    ) -> Self {
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
        if self.render_type == InterpolableRenderType::GlobalContext {
            self.balance_single_quotes();
        }
        while let Some(string) = self.strings.pop_front() {
            result.push(self.translate_escaped_string(string));
            if let Some(translated) = self.interps.pop_front() {
                // Quotes inside of interpolable strings are not necessary
                if let FragmentKind::Interpolable(mut interpolable) = translated {
                    interpolable =
                        interpolable.with_render_type(InterpolableRenderType::GlobalContext);
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
        let mut in_double_quotes = false;
        let total_strings = self.strings.len();
        let total_interps = self.interps.len();
        let mut reopen_single_quotes = false;

        for (idx, s) in self.strings.iter_mut().enumerate() {
            // If previous chunk left us inside quotes, reopen at the start.
            if reopen_single_quotes {
                s.insert_str(0, "\"'");
                reopen_single_quotes = false;
            }

            scan_quote_state(s, &mut in_single_quotes, &mut in_double_quotes);

            let has_more_strings = idx + 1 < total_strings;
            let has_more_interps = idx < total_interps;

            if in_single_quotes && (has_more_strings || has_more_interps) {
                // Close the chunk locally so each piece is balanced.
                s.push_str("'\"");
                in_single_quotes = false;
                in_double_quotes = true;
                reopen_single_quotes = true;
            }
        }
    }

    fn translate_escaped_string(&self, string: String) -> String {
        let chars = string.chars();
        let mut result = String::new();
        for c in chars {
            match self.render_type {
                InterpolableRenderType::StringLiteral => match c {
                    '"' => result += r#"\""#,
                    '$' => result += r#"\$"#,
                    '`' => result += r#"\`"#,
                    '\\' => result += r#"\\"#,
                    '!' => result += r#""'!'""#,
                    _ => result.push(c),
                },
                InterpolableRenderType::GlobalContext => result.push(c),
            }
        }
        result
    }
}

/// Scans a string to determine the quoting state, updating the state flags.
/// Returns `true` if the single-quote state was toggled.
fn scan_quote_state(s: &str, in_single_quotes: &mut bool, in_double_quotes: &mut bool) {
    let mut backslashes = 0;

    for b in s.bytes() {
        match b {
            b'\\' => backslashes += 1,
            b'"' => {
                if !*in_single_quotes && backslashes % 2 == 0 {
                    *in_double_quotes = !*in_double_quotes;
                }
                backslashes = 0;
            }
            b'\'' => {
                if !*in_double_quotes && backslashes % 2 == 0 {
                    *in_single_quotes = !*in_single_quotes;
                }
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
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
        assert_eq!(
            i_str.translate_escaped_string(r#"hello"#.to_string()),
            r#"hello"#
        );
        assert_eq!(i_str.translate_escaped_string(r#"\"#.to_string()), r#"\\"#);
        assert_eq!(i_str.translate_escaped_string(r#"""#.to_string()), r#"\""#);
        assert_eq!(i_str.translate_escaped_string(r#"'"#.to_string()), r#"'"#);
        assert_eq!(i_str.translate_escaped_string(r#"$"#.to_string()), r#"\$"#);
        assert_eq!(
            i_str.translate_escaped_string(r#"\$"#.to_string()),
            r#"\\\$"#
        );
        assert_eq!(i_str.translate_escaped_string(r#"{"#.to_string()), r#"{"#);
        assert_eq!(i_str.translate_escaped_string(r#"`"#.to_string()), r#"\`"#);
        assert_eq!(
            i_str.translate_escaped_string(r#"!"#.to_string()),
            r#""'!'""#
        );
        assert_eq!(
            i_str.translate_escaped_string(r#"\ "#.to_string()),
            r#"\\ "#
        );
        assert_eq!(
            i_str.translate_escaped_string(r#"${var}"#.to_string()),
            r#"\${var}"#
        );

        // Test GlobalContext translation
        let i_glo = create_interpolable(InterpolableRenderType::GlobalContext);
        assert_eq!(
            i_glo.translate_escaped_string(r#"hello"#.to_string()),
            r#"hello"#
        );
        assert_eq!(i_glo.translate_escaped_string(r#"\a"#.to_string()), r#"\a"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\"#.to_string()), r#"\"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\\"#.to_string()), r#"\\"#);
        assert_eq!(i_glo.translate_escaped_string(r#"""#.to_string()), r#"""#);
        assert_eq!(i_glo.translate_escaped_string(r#"'"#.to_string()), r#"'"#);
        assert_eq!(i_glo.translate_escaped_string(r#"$"#.to_string()), r#"$"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\$"#.to_string()), r#"\$"#);
        assert_eq!(i_glo.translate_escaped_string(r#"{"#.to_string()), r#"{"#);
        assert_eq!(i_glo.translate_escaped_string(r#"!"#.to_string()), r#"!"#);
        assert_eq!(
            i_glo.translate_escaped_string(r#"basename `pwd`"#.to_string()),
            r#"basename `pwd`"#
        );
        assert_eq!(i_glo.translate_escaped_string(r#"\ "#.to_string()), r#"\ "#);
    }

    #[test]
    fn test_toggles_single_quote_state() {
        let mut dq = false;
        let mut sq = false;
        scan_quote_state(r#"foo"#, &mut sq, &mut dq);
        scan_quote_state(r#"foo\'bar"#, &mut sq, &mut dq);
        scan_quote_state(r#"foo'bar"#, &mut sq, &mut dq);
        // even number of backslashes before quote -> not escaped
        scan_quote_state(r#"foo\\\\'bar"#, &mut sq, &mut dq);
        scan_quote_state(r#"'"#, &mut sq, &mut dq);
        scan_quote_state(r#"'\"'"#, &mut sq, &mut dq);
        scan_quote_state(r#"'''"#, &mut sq, &mut dq);
        scan_quote_state(r#""'""#, &mut sq, &mut dq);

        sq = false;
        dq = false;
        scan_quote_state(r#" '" "#, &mut sq, &mut dq);
        assert!(sq);
        assert!(!dq);
        scan_quote_state(r#" '" "#, &mut sq, &mut dq);
        assert!(!sq);
        assert!(dq);
        scan_quote_state(r#" \"'\" "#, &mut sq, &mut dq);
        assert!(!sq);
        assert!(dq);
        scan_quote_state(r#" " "#, &mut sq, &mut dq);
        assert!(!sq);
        assert!(!dq);
        scan_quote_state(r#" ' "#, &mut sq, &mut dq);
        assert!(sq);
        assert!(!dq);
        scan_quote_state(r#" \' "#, &mut sq, &mut dq);
        assert!(sq);
        assert!(!dq);
        scan_quote_state(r#" \" "#, &mut sq, &mut dq);
        assert!(sq);
        assert!(!dq);
        scan_quote_state(r#" " "#, &mut sq, &mut dq);
        assert!(sq);
        assert!(!dq);
        scan_quote_state(r#" '"' "#, &mut sq, &mut dq);
        assert!(!sq);
        assert!(dq);
    }
}
