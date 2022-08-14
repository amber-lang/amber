use std::collections::VecDeque;
use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use crate::modules::expression::expr::Expr;

pub mod bool;
pub mod number;
pub mod text;
pub mod null;

pub fn parse_interpolated_region(meta: &mut ParserMetadata, letter: char) -> Result<(Vec<String>, Vec<Expr>), ErrorDetails> {
    let mut strings = vec![];
    let mut interps = vec![];
    // Handle full string
    if let Ok(word) = token_by(meta, |word| word.starts_with(letter) && word.ends_with(letter) && word.len() > 1) {
        let stripped = word.chars().take(word.len() - 1).skip(1).collect::<String>();
        strings.push(stripped.clone());
        Ok((strings, interps))
    }
    else {
        let mut is_interp = false;
        // Initialize string
        let start = token_by(meta, |word| word.starts_with(letter))?;
        strings.push(start.chars().skip(1).collect::<String>());
        // Factor rest of the interpolation
        while let Some(token) = meta.get_current_token() {
            // Track interpolations
            match token.word.as_str() {
                "{" => is_interp = true,
                "}" => is_interp = false,
                // Manage inserting strings and intrpolations
                _ => if is_interp {
                    let mut expr = Expr::new();
                    syntax(meta, &mut expr)?;
                    interps.push(expr);
                    // TODO: [H50] In the next release of Heraclitus
                    // Change this line to `meta.offset_index(-1)`
                    meta.set_index(meta.get_index() - 1);
                }
                else {
                    strings.push(token.word.clone());
                    if token.word.ends_with(letter) {
                        meta.increment_index();
                        // Right trim the symbol
                        let trimmed = strings.last().unwrap()
                            .chars().take(token.word.len() - 1).collect::<String>();
                        // replace the last string
                        *strings.last_mut().unwrap() = trimmed;
                        return Ok((strings, interps))
                    }
                }
            }
            meta.increment_index();
        }
        Err(ErrorDetails::from_metadata(meta))
    }
}

fn translate_escaped_string(string: String) -> String {
    let mut chars = string.chars().peekable();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                match chars.peek() {
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
                        result.push('\'');
                        chars.next();
                    },
                    Some('\"') => {
                        result.push('\"');
                        chars.next();
                    },
                    Some('\\') => {
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

pub fn translate_interpolated_region(strings: Vec<String>, interps: Vec<String>) -> String {
    // TODO: [A15] Fix issues related to the escaping
    let mut result = vec![];
    let mut interps = VecDeque::from_iter(interps);
    let mut strings = VecDeque::from_iter(strings);
    let mut is_even = false;
    loop {
        let value = if is_even { interps.pop_front() } else { strings.pop_front() };
        match value {
            Some(translated) => {
                if is_even {
                    result.push(translated);
                } else {
                    result.push(translate_escaped_string(translated));
                }
            },
            None => break
        }
        is_even = !is_even;
    }
    result.join("")
}