use include_dir::{include_dir, Dir};
pub const STDLIB: Dir = include_dir!("src/std");

pub fn resolve_std<T: Into<String>>(path: T) -> Option<String> {
    let path = path.into();
    let path = if path == "std" { "main".to_string() } else { path };

    if let Some(module) = STDLIB.get_file(path + ".ab") {
        let module = module.contents_utf8().unwrap().to_string();
        Some(module)
    } else {
        None
    }
}