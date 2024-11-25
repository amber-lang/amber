use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct FileMeta {
    pub is_import: bool,
    pub source: FileSource
}

impl FileMeta {
    pub fn file(is_import: bool) -> Self {
        Self {
            is_import,
            source: FileSource::File
        }
    }

    pub fn stdlib(is_import: bool) -> Self {
        Self {
            is_import,
            source: FileSource::Stdlib
        }
    }

    pub fn stream(is_import: bool) -> Self {
        Self {
            is_import,
            source: FileSource::Stream
        }
    }
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