use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::PathBuf};
use std::time::SystemTime;

use heraclitus_compiler::prelude::Token;
use serde::{Deserialize, Serialize};

use crate::compiler::cache::GIT_HASH;
use crate::compiler::file_source::{FileMeta, FileSource};
use crate::stdlib;

use super::home_cache;

pub const FILE_EXT: &'static str = "ab.tk.c";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PretokenizedFile {
    pub tokens: Vec<Token>,
    pub mtime: Option<SystemTime>,
    pub len: u64,
    pub amber_git_hash: String,
    pub file_meta: FileMeta
}

impl PretokenizedFile {
    pub fn load_for(filename: impl Into<PathBuf>, file_meta: FileMeta) -> Result<Option<PretokenizedFile>, Box<dyn std::error::Error>> {
        if file_meta.source.cache_disabled() {
            return Ok(None)
        }

        let filename: PathBuf = filename.into();

        match file_meta.source {
            FileSource::File => {
                if ! filename.is_file() {
                    return Ok(None)
                }
            },
            FileSource::Stdlib => {
                if Self::get_stdlib(&filename).is_none() {
                    return Ok(None)
                }
            },
            _ => unimplemented!()
        }

        let cache_filename = Self::get_path(&filename, file_meta);

        if let Some(cache_filename) = cache_filename {
            
            if ! cache_filename.is_file() {
                return Ok(None)
            }

            let parsed = Self::try_from(fs::read(&cache_filename)?)?;

            if parsed.validate(&filename, file_meta) {
                Ok(Some(parsed))
            } else {
                Ok(None)
            }
            
        } else {
            Err(String::from("Couldn't get cache filename!").into())
        }
    }

    pub fn save(filename: impl Into<PathBuf>, file_meta: FileMeta, tokens: Vec<Token>) -> Result<(), Box<dyn std::error::Error>> {
        let filename: PathBuf = filename.into();

        let (mtime, len) = match file_meta.source {
            FileSource::File => {
                let file = File::open(&filename)?;
                let meta = file.metadata()?;
                (Some(meta.modified()?), meta.len())
            },
            FileSource::Stdlib => {
                if let Some(file) = Self::get_stdlib(&filename) {
                    
                    (None, file.len() as u64)

                } else {
                    return Err(String::from("Couldn't find stdlib file").into())
                }
            }
            _ => unimplemented!()
        };
        
        let cache = Self {
            tokens,
            mtime,
            len,
            amber_git_hash: GIT_HASH.into(),
            file_meta
        };
        
        cache.save_to_file(&filename, file_meta)?;

        Ok(())
    }

    fn get_stdlib(filename: impl Into<PathBuf>) -> Option<String> {
        let filename = filename.into();
        stdlib::resolve(filename.components().last().unwrap().as_os_str().to_str().unwrap())
    }

    fn save_to_file(&self, filename: impl Into<PathBuf>, file_meta: FileMeta) -> Result<(), Box<dyn std::error::Error>> {
        let filename: PathBuf = filename.into();

        if let Some(cache_file) = Self::get_path(&filename, file_meta) {
            let serialized: Vec<u8> = self.try_into()?;
            
            fs::write(&cache_file, serialized).map_err(|x| format!("Cannot write to {cache_file:?}: {x}"))?;

            #[cfg(unix)]
            fs::set_permissions(&filename, Permissions::from_mode(0o700)).map_err(|x| format!("Cannot set perms to {cache_file:?}: {x}"))?;
            
            Ok(())
        } else {
            Err(String::from("Couldn't get path to saved directory").into())
        }
    }

    fn get_path(filename: impl Into<PathBuf>, file_meta: FileMeta) -> Option<PathBuf> {
        let filename: PathBuf = filename.into();
        let filename: PathBuf = filename.to_str().unwrap().replace('/', "_").replace(":", "_").into();

        match file_meta.source {
            FileSource::Stdlib => {
                home_cache().map(|mut x| {
                    x.push(&filename);
                    x.set_extension(FILE_EXT);
                    x
                })
            },
            FileSource::File => {
                let mut filename = filename.clone();
                filename.set_extension(FILE_EXT);
                Some(filename)
            },
            _ => unimplemented!()
        }
    }

    fn validate(&self, filename: impl Into<PathBuf>, file_meta: FileMeta) -> bool {
        assert_eq!(self.file_meta, file_meta);
        let filename: PathBuf = filename.into();
        
        if file_meta.source.cache_disabled() {
            return false;
        }
        if self.amber_git_hash != GIT_HASH {
            return false;
        }

        match file_meta.source {
            FileSource::File => {
                if let Ok(meta) = File::open(&filename) {
                    if let Ok(meta) = meta.metadata() {
                        if ! meta.is_file() { return false }
                        if meta.len() != self.len { return false }

                        if let Ok(mtime) = meta.modified() {
                            if let Some(smtime) = self.mtime {
                                if mtime != smtime { return false }
                            }
                        }
                    }
                }
            },
            FileSource::Stdlib => {
                if let Some(file) = stdlib::resolve(filename.to_str().unwrap()) {
                    if file.len() != self.len as usize {
                        return false;
                    }
                    if self.mtime.is_some() {
                        return false;
                    }
                }
            },
            _ => unimplemented!()
        }
        
        true
    }
}

impl TryInto<Vec<u8>> for &PretokenizedFile {
    type Error = String;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        Ok(rmp_serde::to_vec(&self).map_err(|x| x.to_string())?)
    }
}

impl TryFrom<Vec<u8>> for PretokenizedFile {
    type Error = String;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(rmp_serde::from_slice(&value).map_err(|x| x.to_string())?)
    }
}
