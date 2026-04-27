use std::fs;
use std::path::PathBuf;

/// Adds all amber files (.ab extension) to from the directory to the vector. The search is recursive.
///
/// * `dir` Directory to search
/// * `files` Vector to add results to
pub fn find_amber_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_amber_files(&path, files)?;
            } else if let Some(ext) = path.extension() {
                if ext == "ab" {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}