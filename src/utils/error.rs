use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;

pub fn get_error_logger(meta: &mut ParserMetadata, details: ErrorDetails) -> Logger {
    let path = meta.path.clone();
    let code = meta.code.clone();
    Logger::new_err_with_details(path, code, details)
}

pub fn get_warn_logger(meta: &mut ParserMetadata, details: ErrorDetails) -> Logger {
    let path = meta.path.clone();
    let code = meta.code.clone();
    Logger::new_warn_with_details(path, code, details)
}