use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use crate::modules::expression::expr::Expr;

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
            InterpolatedRegionType::Command => '$'
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

pub fn parse_interpolated_region(meta: &mut ParserMetadata, interpolated_type: &InterpolatedRegionType) -> Result<(Vec<String>, Vec<Expr>), Failure> {
    let mut strings = vec![];
    let mut interps = vec![];
    let letter = interpolated_type.to_char();
    // Handle full string
    if let Ok(word) = token_by(meta, |word| {
        word.starts_with(letter)
        && word.ends_with(letter)
        && word.len() > 1
        && !is_escaped(word, letter)
    }) {
        let stripped = word.chars().take(word.chars().count() - 1).skip(1).collect::<String>();
        strings.push(parse_escaped_string(stripped, interpolated_type));
        Ok((strings, interps))
    }
    else {
        let mut is_interp = false;
        // Initialize string
        let start = token_by(meta, |word| word.starts_with(letter))?;
        strings.push(parse_escaped_string(start.chars().skip(1).collect::<String>(), interpolated_type));
        // Factor rest of the interpolation
        while let Some(tok) = meta.get_current_token() {
            // Track interpolations
            match tok.word.as_str() {
                "{" => is_interp = true,
                "}" => is_interp = false,
                // Manage inserting strings and intrpolations
                _ => if is_interp {
                    let mut expr = Expr::new();
                    syntax(meta, &mut expr)?;
                    interps.push(expr);
                    meta.offset_index(-1);
                }
                else {
                    strings.push(parse_escaped_string(tok.word.clone(), interpolated_type));
                    if tok.word.ends_with(letter) && !is_escaped(&tok.word, letter) {
                        meta.increment_index();
                        // Right trim the symbol
                        let trimmed = strings.last().unwrap()
                            .chars().take(parse_escaped_string(tok.word, interpolated_type).chars().count() - 1).collect::<String>();
                        // replace the last string
                        *strings.last_mut().unwrap() = trimmed;
                        return Ok((strings, interps))
                    }
                }
            }
            meta.increment_index();
        }
        Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_escaped_string() {
        let text_type = InterpolatedRegionType::Text;
        let command_type = InterpolatedRegionType::Command;

        // Test text parsing
        assert_eq!(parse_escaped_string("hello".to_string(), &text_type), "hello");
        assert_eq!(parse_escaped_string("\n".to_string(), &text_type), "\n");
        assert_eq!(parse_escaped_string("\t".to_string(), &text_type), "\t");
        assert_eq!(parse_escaped_string("\r".to_string(), &text_type), "\r");
        assert_eq!(parse_escaped_string("\0".to_string(), &text_type), "\0");
        assert_eq!(parse_escaped_string(r#"\\"#.to_string(), &text_type), r#"\"#);
        assert_eq!(parse_escaped_string(r#"'"#.to_string(), &text_type), r#"'"#);
        assert_eq!(parse_escaped_string(r#"\""#.to_string(), &text_type), r#"""#);
        assert_eq!(parse_escaped_string(r#"$"#.to_string(), &text_type), r#"$"#);
        assert_eq!(parse_escaped_string(r#"\\$"#.to_string(), &text_type), r#"\$"#);
        assert_eq!(parse_escaped_string(r#"\{"#.to_string(), &text_type), r#"{"#);
        assert_eq!(parse_escaped_string(r#"\\ "#.to_string(), &text_type), r#"\ "#);
        assert_eq!(parse_escaped_string(r#"$\{var}"#.to_string(), &text_type), r#"${var}"#);
        assert_eq!(parse_escaped_string(r#"\\$\{var}"#.to_string(), &text_type), r#"\${var}"#);

        // Test command parsing
        assert_eq!(parse_escaped_string("hello".to_string(), &command_type), "hello");
        assert_eq!(parse_escaped_string("\n".to_string(), &command_type), "\n");
        assert_eq!(parse_escaped_string("\t".to_string(), &command_type), "\t");
        assert_eq!(parse_escaped_string("\r".to_string(), &command_type), "\r");
        assert_eq!(parse_escaped_string("\0".to_string(), &command_type), "\0");
        assert_eq!(parse_escaped_string(r#"\\"#.to_string(), &command_type), r#"\"#);
        assert_eq!(parse_escaped_string(r#"""#.to_string(), &command_type), r#"""#);
        assert_eq!(parse_escaped_string(r#"\""#.to_string(), &command_type), r#"\""#);
        assert_eq!(parse_escaped_string(r#"'"#.to_string(), &command_type), r#"'"#);
        assert_eq!(parse_escaped_string(r#"\'"#.to_string(), &command_type), r#"\'"#);
        assert_eq!(parse_escaped_string(r#"\$"#.to_string(), &command_type), r#"$"#);
        assert_eq!(parse_escaped_string(r#"\\\$"#.to_string(), &command_type), r#"\$"#);
        assert_eq!(parse_escaped_string(r#"\{"#.to_string(), &command_type), r#"{"#);
        assert_eq!(parse_escaped_string(r#"basename `pwd`"#.to_string(), &command_type), r#"basename `pwd`"#);
        assert_eq!(parse_escaped_string(r#"\$\{var}"#.to_string(), &command_type), r#"${var}"#);
        assert_eq!(parse_escaped_string(r#"\\\$\{var}"#.to_string(), &command_type), r#"\${var}"#);
    }
}
