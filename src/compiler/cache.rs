use std::{fs, path::PathBuf};

const GIT_HASH: &'static str = env!("GIT_HASH");

pub mod parse;
pub mod tokenize;

pub fn home_cache() -> Option<PathBuf> {
    if let Some(mut home) = home::home_dir() {
        home.push(".cache");
        home.push("amber");
        if ! home.is_dir() {
            fs::create_dir_all(&home).expect("Couldn't create ~/.cache/amber");
        }
        Some(home)
    } else {
        None
    }
}
