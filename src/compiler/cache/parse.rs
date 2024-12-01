use std::{error::Error, fs::{self, File, Metadata, Permissions}, os::unix::fs::PermissionsExt, path::PathBuf, time::SystemTime};
use serde::{Serialize, Deserialize};

use crate::{modules::block::Block, utils::ParserMetadata};

use super::GIT_HASH;

pub const FILE_EXT: &'static str = "ab.pr.c";

#[derive(Debug, Serialize, Deserialize)]
pub struct PreparsedFile {
    pub block: Block,
    pub meta: ParserMetadata,
    pub mtime: SystemTime,
    pub len: u64,
    pub amber_git_hash: String
}

impl PreparsedFile {
    pub fn load_for<T: Into<PathBuf>>(filename: T) -> Result<Option<PreparsedFile>, Box<dyn Error>> {
        let mut filename: PathBuf = filename.into();
        if ! filename.is_file() {
            return Ok(None);
        }

        let meta = File::open(&filename)?;
        let meta = meta.metadata()?;

        filename.set_extension(FILE_EXT); // amber compiled
        if ! filename.is_file() {
            return Ok(None);
        }
        
        let preparsed = fs::read(&filename)?;
        if let Ok(preparsed) = Self::try_from(preparsed) {

            if ! preparsed.validate(&meta) {
                fs::remove_file(&filename)?;
                return Ok(None);
            }

            Ok(Some(preparsed))
        } else {
            Ok(None)
        }
    }

    pub fn save<T: Into<PathBuf>>(filename: T, block: Block, pmeta: &ParserMetadata) -> Result<Self, Box<dyn Error>> {
        let filename: PathBuf = filename.into();
        
        let meta = File::open(&filename)?;
        let meta = meta.metadata()?;

        let preparsed = PreparsedFile {
            block,
            meta: pmeta.clone(),
            mtime: meta.modified()?,
            len: meta.len(),
            amber_git_hash: GIT_HASH.to_string()
        };

        preparsed.save_to_file(filename)?;

        Ok(preparsed)
    }

    fn save_to_file<T: Into<PathBuf>>(&self, filename: T) -> Result<(), Box<dyn Error>> {
        let mut filename: PathBuf = filename.into();
        filename.set_extension(FILE_EXT);

        let serialized: Vec<u8> = self.try_into()?;
        fs::write(&filename, serialized)?;

        #[cfg(unix)]
        fs::set_permissions(&filename, Permissions::from_mode(0o700)).map_err(|x| format!("Cannot set perms to {filename:?}: {x}"))?;

        Ok(())
    }
    
    fn validate(self: &Self, meta: &Metadata) -> bool {
        match meta.modified() {
            Ok(v) => {
                v == self.mtime
                    && meta.len() == self.len
                    && self.amber_git_hash == GIT_HASH
            },
            Err(_) => false
        }
    }
}

impl TryInto<Vec<u8>> for &PreparsedFile {
    type Error = String;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        Ok(rmp_serde::to_vec(&self).map_err(|x| x.to_string())?)
    }
}

impl TryFrom<Vec<u8>> for PreparsedFile {
    type Error = String;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(rmp_serde::from_slice(&value).map_err(|x| x.to_string())?)
    }
}
