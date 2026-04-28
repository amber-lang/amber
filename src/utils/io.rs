use std::path::PathBuf;
use walkdir::WalkDir;

/// Adds all amber files (.ab extension) from the directory to the vector. The search is recursive.
///
/// * `dir` Directory to search
/// * `files` Vector to add results to
pub fn find_amber_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in WalkDir::new(dir).follow_links(true) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "ab" {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
    }
    Ok(())
}