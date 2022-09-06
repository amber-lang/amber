use heraclitus_compiler::prelude::*;

use crate::utils::{error::get_error_logger, ParserMetadata};

pub mod module;
pub mod compute;

pub fn check_all_blocks(meta: &mut ParserMetadata) {
    let mut stack = 0;
    for token in meta.expr.iter() {
        match token.word.as_str() {
            "{" => stack += 1,
            "}" => stack -= 1,
            _ => ()
        }
        if stack < 0 {
            get_error_logger(meta, ErrorDetails::from_token_option(Some(token.clone())))
                .attach_comment("There are too many closing brackets")
                .show().exit();
        }
    }
}