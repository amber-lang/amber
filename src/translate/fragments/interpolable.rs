use std::collections::VecDeque;

use super::fragment::{FragmentKind, FragmentRenderable};
use crate::utils::TranslateMetadata;

/// Represents a region that can be interpolated. Similarly to what Heraclitus returns when parsing a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolableRenderType {
    /// This should be rendered to Bash's double quoted string
    StringLiteral,
    /// This should be rendered to Bash's global context expression (command)
    GlobalContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolablePart {
    String(String),
    Interp(FragmentKind),
}

impl InterpolablePart {
    pub fn is_running_command(&self) -> bool {
        match self {
            InterpolablePart::String(_) => false,
            InterpolablePart::Interp(frag) => frag.is_running_command(),
        }
    }

    pub fn is_mutating(&self) -> bool {
        match self {
            InterpolablePart::String(_) => false,
            InterpolablePart::Interp(frag) => frag.is_mutating(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpolableFragment {
    pub parts: VecDeque<InterpolablePart>,
    pub render_type: InterpolableRenderType,
    pub quoted: bool,
}

impl InterpolableFragment {
    pub fn new(parts: Vec<InterpolablePart>, render_type: InterpolableRenderType) -> Self {
        InterpolableFragment {
            parts: VecDeque::from(parts),
            render_type,
            quoted: true,
        }
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
        for part in std::mem::take(&mut self.parts) {
            match part {
                InterpolablePart::String(s) => result.push(self.translate_escaped_string(&s)),
                InterpolablePart::Interp(frag) => {
                    let rendered = match frag {
                        FragmentKind::Interpolable(mut interpolable) => {
                            interpolable.render_type = InterpolableRenderType::GlobalContext;
                            interpolable.quoted = false;
                            interpolable.to_string(meta)
                        }
                        _ => frag.to_string(meta),
                    };
                    result.push(rendered);
                }
            }
        }
        result.join("")
    }

    fn balance_single_quotes(&mut self) {
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;
        let total_parts = self.parts.len();
        let mut reopen_single_quotes = false;

        for (idx, part) in self.parts.iter_mut().enumerate() {
            if let InterpolablePart::String(s) = part {
                // If previous chunk left us inside quotes, reopen at the start.
                if reopen_single_quotes {
                    s.insert_str(0, "\"'");
                    reopen_single_quotes = false;
                }
                scan_quote_state(s, &mut in_single_quotes, &mut in_double_quotes);

                let has_more_parts = idx + 1 < total_parts;

                if in_single_quotes && has_more_parts {
                    // Close the chunk locally so each piece is balanced.
                    s.push_str("'\"");
                    in_single_quotes = false;
                    in_double_quotes = true;
                    reopen_single_quotes = true;
                }
            }
        }
        if reopen_single_quotes {
            self.parts.push_back(InterpolablePart::String("\"".to_string()));
        }
    }

    fn translate_escaped_string(&self, string: &str) -> String {
        let mut result = String::new();
        for c in string.chars() {
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
        let render_type = self.render_type;
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
        InterpolableFragment::new(vec![], render_type)
    }

    #[test]
    fn test_translate_escaped_string() {
        // Test StringLiteral translation
        let i_str = create_interpolable(InterpolableRenderType::StringLiteral);
        assert_eq!(i_str.translate_escaped_string(r#"hello"#), r#"hello"#);
        assert_eq!(i_str.translate_escaped_string(r#"\"#), r#"\\"#);
        assert_eq!(i_str.translate_escaped_string(r#"""#), r#"\""#);
        assert_eq!(i_str.translate_escaped_string(r#"'"#), r#"'"#);
        assert_eq!(i_str.translate_escaped_string(r#"$"#), r#"\$"#);
        assert_eq!(i_str.translate_escaped_string(r#"\$"#), r#"\\\$"#);
        assert_eq!(i_str.translate_escaped_string(r#"{"#), r#"{"#);
        assert_eq!(i_str.translate_escaped_string(r#"`"#), r#"\`"#);
        assert_eq!(i_str.translate_escaped_string(r#"!"#), r#""'!'""#);
        assert_eq!(i_str.translate_escaped_string(r#"\ "#), r#"\\ "#);
        assert_eq!(i_str.translate_escaped_string(r#"${var}"#), r#"\${var}"#);

        // Test GlobalContext translation
        let i_glo = create_interpolable(InterpolableRenderType::GlobalContext);
        assert_eq!(i_glo.translate_escaped_string(r#"hello"#), r#"hello"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\a"#), r#"\a"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\"#), r#"\"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\\"#), r#"\\"#);
        assert_eq!(i_glo.translate_escaped_string(r#"""#), r#"""#);
        assert_eq!(i_glo.translate_escaped_string(r#"'"#), r#"'"#);
        assert_eq!(i_glo.translate_escaped_string(r#"$"#), r#"$"#);
        assert_eq!(i_glo.translate_escaped_string(r#"\$"#), r#"\$"#);
        assert_eq!(i_glo.translate_escaped_string(r#"{"#), r#"{"#);
        assert_eq!(i_glo.translate_escaped_string(r#"!"#), r#"!"#);
        assert_eq!(
            i_glo.translate_escaped_string(r#"basename `pwd`"#),
            r#"basename `pwd`"#
        );
        assert_eq!(i_glo.translate_escaped_string(r#"\ "#), r#"\ "#);
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
