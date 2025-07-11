use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use crate::modules::expression::expr::Expr;

pub mod bool;
pub mod number;
pub mod text;
pub mod null;
pub mod array;
pub mod status;

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

fn validate_escape_sequences(meta: &mut ParserMetadata, string_content: &str, tok: Option<&Token>) {
    let mut chars = string_content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_char) = chars.peek() {
                match next_char {
                    // Valid escape sequences
                    'n' | 't' | 'r' | '0' | '{' | '$' | '\'' | '"' | '\\' => {
                        chars.next(); // consume the valid escape character
                    }
                    // Invalid escape sequences
                    _ => {
                        let warning_msg = format!("Invalid escape sequence '\\{next_char}'");
                        let message = if let Some(token) = tok {
                            Message::new_warn_at_token(meta, Some(token.clone()))
                                .message(warning_msg)
                                .comment("Only these escape sequences are supported: \\n, \\t, \\r, \\0, \\{, \\$, \\', \\\", \\\\")
                        } else {
                            Message::new_warn_msg(warning_msg)
                                .comment("Only these escape sequences are supported: \\n, \\t, \\r, \\0, \\{, \\$, \\', \\\", \\\\")
                        };
                        meta.add_message(message);
                        chars.next(); // consume the invalid escape character
                    }
                }
            }
        }
    }
}

pub fn parse_interpolated_region(meta: &mut ParserMetadata, letter: char) -> Result<(Vec<String>, Vec<Expr>), Failure> {
    let mut strings = vec![];
    let mut interps = vec![];
    // Handle full string
    if let Ok(word) = token_by(meta, |word| {
        word.starts_with(letter)
        && word.ends_with(letter)
        && word.len() > 1
        && !is_escaped(word, letter)
    }) {
        let stripped = word.chars().take(word.chars().count() - 1).skip(1).collect::<String>();
        // Validate escape sequences in the string content
        let current_token = meta.get_current_token();
        validate_escape_sequences(meta, &stripped, current_token.as_ref());
        strings.push(stripped);
        Ok((strings, interps))
    }
    else {
        let mut is_interp = false;
        // Initialize string
        let start = token_by(meta, |word| word.starts_with(letter))?;
        let start_content = start.chars().skip(1).collect::<String>();
        // Validate escape sequences in the initial string part
        let current_token = meta.get_current_token();
        validate_escape_sequences(meta, &start_content, current_token.as_ref());
        strings.push(start_content);
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
                    let string_content = tok.word.clone();
                    if string_content.ends_with(letter) && !is_escaped(&string_content, letter) {
                        meta.increment_index();
                        // Right trim the symbol
                        let trimmed = string_content
                            .chars().take(string_content.chars().count() - 1).collect::<String>();
                        // Validate escape sequences in this string part
                        validate_escape_sequences(meta, &trimmed, Some(&tok));
                        // replace the last string
                        strings.push(trimmed);
                        return Ok((strings, interps))
                    } else {
                        // Validate escape sequences in this string part
                        validate_escape_sequences(meta, &string_content, Some(&tok));
                        strings.push(string_content);
                    }
                }
            }
            meta.increment_index();
        }
        Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
    }
}
