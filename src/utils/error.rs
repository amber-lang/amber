use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;

pub fn get_error_logger(meta: &ParserMetadata, details: ErrorDetails) -> Logger {
    Logger::new_err_with_trace(meta, &[details])
}

pub fn get_warn_logger(meta: &ParserMetadata, details: ErrorDetails) -> Logger {
    Logger::new_warn_with_trace(meta, &[details])
}