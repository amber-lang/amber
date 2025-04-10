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
