use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct FileMeta {
    pub is_import: bool,
    pub source: FileSource
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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