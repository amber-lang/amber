use std::{env, fs, path::PathBuf};

#[cfg(not(debug_assertions))]
fn get_install_dir() -> PathBuf {
    let path = env::var("STD_PATH").expect("STD_PATH not set");

    PathBuf::from(path)
}

#[cfg(debug_assertions)]
fn get_install_dir() -> PathBuf {
    let path = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    PathBuf::from(path).join("resources/std")
}

pub fn resolve<T: Into<String>>(path: T) -> Option<String> {
    let path = get_install_dir().join(path.into() + ".ab");

    if !path.is_file() {
        return None;
    }

    if let Ok(contents) = fs::read_to_string(path) {
        Some(contents)
    } else {
        None
    }
}
