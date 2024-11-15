use std::{error::Error, fs::{self, File, Metadata}, path::PathBuf, time::SystemTime};
use serde::{Serialize, Deserialize};

use crate::{modules::block::Block, utils::ParserMetadata};

use super::GIT_HASH;

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

        filename.set_extension("abc"); // amber compiled
        if ! filename.is_file() {
            return Ok(None);
        }
        
        let preparsed = fs::read(&filename)?;
        if let Ok(preparsed) = rmp_serde::from_slice::<PreparsedFile>(&preparsed) {

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
        filename.set_extension("abc");

        let serialized = rmp_serde::to_vec(self)?;
        fs::write(filename, serialized)?;

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
