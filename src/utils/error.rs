use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;

fn get_full_trace(meta: &ParserMetadata, details: ErrorDetails) -> Vec<ErrorDetails> {
    let mut trace = meta.trace.clone();
    trace.push(details);
    trace
}

pub fn get_error_logger(meta: &ParserMetadata, details: ErrorDetails) -> Logger {
    Logger::new_err_with_trace(meta, &get_full_trace(meta, details))
}

pub fn get_warn_logger(meta: &ParserMetadata, details: ErrorDetails) -> Logger {
    Logger::new_warn_with_trace(meta, &get_full_trace(meta, details))
}