use std::{env, fs, path::PathBuf};

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

#[cfg(not(debug_assertions))]
fn get_install_dir() -> PathBuf {
    let exec_path = env::current_exe().expect("Could not fetch executable file path.");

    PathBuf::from(exec_path.read_link().unwrap_or(exec_path).parent().unwrap()).join("std")
}

#[cfg(debug_assertions)]
fn get_install_dir() -> PathBuf {
    let path = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    PathBuf::from(path).join("std")
}
