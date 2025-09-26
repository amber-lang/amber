use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;

pub mod bool;
pub mod number;
pub mod integer;
pub mod text;
pub mod null;
pub mod array;
pub mod status;

fn validate_text_escape_sequences(meta: &mut ParserMetadata, string_content: &str, start_pos: usize, end_pos: usize) {
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
                        let pos = PositionInfo::from_between_tokens(meta, meta.get_token_at(start_pos), meta.get_token_at(end_pos));
                        let message = Message::new_warn_at_position(meta, pos)
                            .message(warning_msg)
                            .comment("Only these escape sequences are supported: \\n, \\t, \\r, \\0, \\{, \\$, \\', \\\", \\\\");
                        meta.add_message(message);
                        chars.next(); // consume the invalid escape character
                    }
                }
            }
        }
    }
}
