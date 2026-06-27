use include_dir::{include_dir, Dir};
pub const STDLIB: Dir = include_dir!("src/std");

pub fn resolve<T: Into<String>>(path: T) -> Option<String> {
    let path = path.into();

    if let Some(module) = STDLIB.get_file(path + ".ab") {
        module.contents_utf8().ok().map(|s| s.to_string())
    } else {
        None
    }
}
