use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;

pub fn get_error_logger(meta: &mut ParserMetadata, mut details: ErrorDetails) -> Logger {
    if let Some(path) = meta.path.clone() {
        if let Ok(location) = details.get_pos_by_file(&path) {
            return Logger::new_err(path, location)
        }
    }
    Logger::new_err_msg(format!("Error at {:?}", details.position))
}