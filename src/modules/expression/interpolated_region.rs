use crate::modules::expression::expr::Expr;
use crate::modules::expression::literal::text::TextPart;
use crate::utils::metadata::ParserMetadata;
use heraclitus_compiler::prelude::*;

/// Represents a literal text or a command.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedRegionType {
    Text,
    Command,
}

impl InterpolatedRegionType {
    pub fn to_char(&self) -> char {
        match self {
            InterpolatedRegionType::Text => '"',
            InterpolatedRegionType::Command => '$',
        }
    }
}

/// Returns true if the words contain an even number of `\`.
fn is_escaped(word: &str, symbol: char) -> bool {
    let mut backslash_count = 0;

    if !word.ends_with(symbol) {
        return false;
    }

    for letter in word.chars().rev().skip(1) {
        if letter == '\\' {
            backslash_count += 1;
        } else {
            break;
        }
    }

    backslash_count % 2 != 0
}

/// Parse Amber code's escaped strings and reterns it.
fn parse_escaped_string(string: String, region_type: &InterpolatedRegionType) -> String {
    let mut chars = string.chars().peekable();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('\n') => {}
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('0') => result.push('\0'),
                Some('{') => result.push('{'),
                Some('"') => {
                    if *region_type == InterpolatedRegionType::Text {
                        result.push('"');
                    } else {
                        result.push(c);
                        continue;
                    }
                }
                Some('$') => {
                    if *region_type == InterpolatedRegionType::Command {
                        result.push('$');
                    } else {
                        result.push(c);
                        continue;
                    }
                }
                _ => {
                    result.push(c);
                    continue;
                }
            }
            chars.next();
        } else {
            result.push(c)
        }
    }
    result
}

fn parse_simple_region(word: &str, interpolated_type: &InterpolatedRegionType) -> Vec<TextPart> {
    let content = &word[1..word.len() - 1];
    vec![TextPart::String(parse_escaped_string(
        content.to_string(),
        interpolated_type,
    ))]
}

fn parse_complex_region(
    meta: &mut ParserMetadata,
    start: String,
    letter: char,
    interpolated_type: &InterpolatedRegionType,
) -> Result<Vec<TextPart>, Failure> {
    let mut parts = vec![];
    let mut is_interp = false;

    parts.push(TextPart::String(parse_escaped_string(
        start[1..].to_string(),
        interpolated_type,
    )));

    while let Some(tok) = meta.get_current_token() {
        match tok.word.as_str() {
            "{" => is_interp = true,
            "}" => is_interp = false,
            _ => {
                if is_interp {
                    let mut expr = Expr::new();
                    syntax(meta, &mut expr)?;
                    parts.push(TextPart::Expr(Box::new(expr)));
                    meta.offset_index(-1);
                } else {
                    if tok.word.ends_with(letter) && !is_escaped(&tok.word, letter) {
                        meta.increment_index();
                        let content = &tok.word[..tok.word.len() - 1];
                        parts.push(TextPart::String(parse_escaped_string(
                            content.to_string(),
                            interpolated_type,
                        )));
                        return Ok(parts);
                    }
                    parts.push(TextPart::String(parse_escaped_string(
                        tok.word.clone(),
                        interpolated_type,
                    )));
                }
            }
        }
        meta.increment_index();
    }

    Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
}

pub fn parse_interpolated_region(
    meta: &mut ParserMetadata,
    interpolated_type: &InterpolatedRegionType,
) -> Result<Vec<TextPart>, Failure> {
    let letter = interpolated_type.to_char();

    if let Ok(word) = token_by(meta, |word| {
        word.starts_with(letter)
            && word.ends_with(letter)
            && word.len() > 1
            && !is_escaped(word, letter)
    }) {
        return Ok(parse_simple_region(&word, interpolated_type));
    }

    let start = token_by(meta, |word| word.starts_with(letter))?;
    parse_complex_region(meta, start, letter, interpolated_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_escaped_string() {
        let text_type = InterpolatedRegionType::Text;
        let command_type = InterpolatedRegionType::Command;

        // Test text parsing
        assert_eq!(
            parse_escaped_string("hello".to_string(), &text_type),
            "hello"
        );
        assert_eq!(parse_escaped_string("\n".to_string(), &text_type), "\n");
        assert_eq!(parse_escaped_string("\t".to_string(), &text_type), "\t");
        assert_eq!(parse_escaped_string("\r".to_string(), &text_type), "\r");
        assert_eq!(parse_escaped_string("\0".to_string(), &text_type), "\0");
        assert_eq!(
            parse_escaped_string(r#"\\"#.to_string(), &text_type),
            r#"\"#
        );
        assert_eq!(parse_escaped_string(r#"'"#.to_string(), &text_type), r#"'"#);
        assert_eq!(
            parse_escaped_string(r#"\""#.to_string(), &text_type),
            r#"""#
        );
        assert_eq!(parse_escaped_string(r#"$"#.to_string(), &text_type), r#"$"#);
        assert_eq!(
            parse_escaped_string(r#"\\$"#.to_string(), &text_type),
            r#"\$"#
        );
        assert_eq!(
            parse_escaped_string(r#"\{"#.to_string(), &text_type),
            r#"{"#
        );
        assert_eq!(
            parse_escaped_string(r#"\\ "#.to_string(), &text_type),
            r#"\ "#
        );
        assert_eq!(
            parse_escaped_string(r#"$\{var}"#.to_string(), &text_type),
            r#"${var}"#
        );
        assert_eq!(
            parse_escaped_string(r#"\\$\{var}"#.to_string(), &text_type),
            r#"\${var}"#
        );

        // Test command parsing
        assert_eq!(
            parse_escaped_string("hello".to_string(), &command_type),
            "hello"
        );
        assert_eq!(parse_escaped_string("\n".to_string(), &command_type), "\n");
        assert_eq!(parse_escaped_string("\t".to_string(), &command_type), "\t");
        assert_eq!(parse_escaped_string("\r".to_string(), &command_type), "\r");
        assert_eq!(parse_escaped_string("\0".to_string(), &command_type), "\0");
        assert_eq!(
            parse_escaped_string(r#"\\"#.to_string(), &command_type),
            r#"\"#
        );
        assert_eq!(
            parse_escaped_string(r#"""#.to_string(), &command_type),
            r#"""#
        );
        assert_eq!(
            parse_escaped_string(r#"\""#.to_string(), &command_type),
            r#"\""#
        );
        assert_eq!(
            parse_escaped_string(r#"'"#.to_string(), &command_type),
            r#"'"#
        );
        assert_eq!(
            parse_escaped_string(r#"\'"#.to_string(), &command_type),
            r#"\'"#
        );
        assert_eq!(
            parse_escaped_string(r#"\$"#.to_string(), &command_type),
            r#"$"#
        );
        assert_eq!(
            parse_escaped_string(r#"\\\$"#.to_string(), &command_type),
            r#"\$"#
        );
        assert_eq!(
            parse_escaped_string(r#"\{"#.to_string(), &command_type),
            r#"{"#
        );
        assert_eq!(
            parse_escaped_string(r#"basename `pwd`"#.to_string(), &command_type),
            r#"basename `pwd`"#
        );
        assert_eq!(
            parse_escaped_string(r#"\$\{var}"#.to_string(), &command_type),
            r#"${var}"#
        );
        assert_eq!(
            parse_escaped_string(r#"\\\$\{var}"#.to_string(), &command_type),
            r#"\${var}"#
        );
    }
}
