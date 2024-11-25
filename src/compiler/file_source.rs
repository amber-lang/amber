use std::default;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct FileMeta {
    pub is_import: bool,
    pub source: FileSource
}

impl Default for FileMeta {
    fn default() -> Self {
        Self {
            is_import: false,
            source: FileSource::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum FileSource {
    #[default]
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