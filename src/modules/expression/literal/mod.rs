use std::collections::VecDeque;
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
        strings.push(stripped);
        Ok((strings, interps))
    }
    else {
        let mut is_interp = false;
        // Initialize string
        let start = token_by(meta, |word| word.starts_with(letter))?;
        strings.push(start.chars().skip(1).collect::<String>());
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
                    strings.push(tok.word.clone());
                    if tok.word.ends_with(letter) && !is_escaped(&tok.word, letter) {
                        meta.increment_index();
                        // Right trim the symbol
                        let trimmed = strings.last().unwrap()
                            .chars().take(tok.word.chars().count() - 1).collect::<String>();
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

fn translate_escaped_string(string: String, is_str: bool) -> String {
    let mut chars = string.chars().peekable();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                // Escape double quotes if in a string
                if is_str {
                    result.push('\\');
                    result.push('\"');
                }
                else {
                    result.push('"');
                }
            },
            symbol @ ('$' | '`') => {
                if is_str {
                    result.push('\\');
                }
                result.push(symbol);
            },
            '!' => {
                if is_str {
                    result += "\"'!'\"";
                } else {
                    result.push('!')
                }
            }
            '\\' => {
                // Escape symbols
                match chars.peek() {
                    Some('\n') => {
                        // We want to escape new line characters
                        if !is_str {
                            chars.next();
                        }
                    }
                    Some('n') => {
                        result.push('\n');
                        chars.next();
                    },
                    Some('t') => {
                        result.push('\t');
                        chars.next();
                    },
                    Some('r') => {
                        result.push('\r');
                        chars.next();
                    },
                    Some('0') => {
                        result.push('\0');
                        chars.next();
                    },
                    Some('\'') => {
                        if is_str {
                            result.push('\'');
                        } else {
                            result.push('\\');
                            result.push('\'');
                        }
                        chars.next();
                    },
                    Some('\"') => {
                        if is_str {
                            result.push('\\');
                            result.push('\"');
                        } else {
                            result.push('\"');
                        }
                        chars.next();
                    },
                    Some('\\') => {
                        if is_str {
                            result.push('\\');
                        }
                        result.push('\\');
                        chars.next();
                    },
                    Some('{') => {
                        result.push('{');
                        chars.next();
                    },
                    Some('$') => {
                        result.push('$');
                        chars.next();
                    },
                    _ => result.push(c)
                }
            },
            _ => result.push(c)
        }
    }
    result
}

pub fn translate_interpolated_region(strings: Vec<String>, interps: Vec<String>, is_str: bool) -> String {
    let mut result = vec![];
    let mut interps = VecDeque::from_iter(interps);
    let mut strings = VecDeque::from_iter(strings);
    let mut is_even = false;
    loop {
        let value = if is_even { interps.pop_front() } else { strings.pop_front() };
        match value {
            Some(translated) => {
                if is_even {
                    if translated.starts_with('\"') && translated.ends_with('\"') {
                        result.push(translated.get(1..translated.len() - 1).unwrap().to_string());
                    } else {
                        result.push(translated);
                    }
                } else {
                    result.push(translate_escaped_string(translated, is_str));
                }
            },
            None => break
        }
        is_even = !is_even;
    }
    result.join("")
}
