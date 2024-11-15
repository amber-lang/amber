use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct FileMeta {
    pub is_import: bool,
    pub source: FileSource
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FileSource {
    File,
    Stdlib,
    Stream
}

impl FileSource {
    pub fn cache_disabled(&self) -> bool {
        match &self {
            Self::Stream => true,
            _ => false
        }
    }
}