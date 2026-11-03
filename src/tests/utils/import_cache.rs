//! Tests for utils/import_cache.rs

use crate::compiler::SourcePath;
use crate::utils::import_cache::ImportCache;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[cfg(test)]
mod import_cache_tests {
    use super::*;

    /// Creates an empty Amber file and returns its source path
    fn source_file(dir: &Path, name: &str) -> SourcePath {
        let path = dir.join(name);
        fs::write(&path, "").unwrap();
        SourcePath::file(path.to_string_lossy()).unwrap()
    }

    #[test]
    fn test_import_cache_new() {
        let cache = ImportCache::new(SourcePath::Stdin, String::from("echo(1)"));
        assert_eq!(cache.files.len(), 1);
        assert_eq!(cache.files[0].path, SourcePath::Stdin);
        assert_eq!(cache.get_code(&SourcePath::Stdin), Some("echo(1)"));
        assert!(cache.import_graph.len() == 1);
    }

    #[test]
    fn test_import_cache_new_with_path() {
        let dir = tempdir().unwrap();
        let main = source_file(dir.path(), "main.ab");
        let cache = ImportCache::new(main.clone(), String::new());
        assert_eq!(cache.files[0].path, main);
    }

    #[test]
    fn test_get_path_id() {
        let dir = tempdir().unwrap();
        let imported = source_file(dir.path(), "imported.ab");
        let missing = SourcePath::StdLib(String::from("std/missing"));
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &imported);

        assert_eq!(cache.get_path_id(&SourcePath::Stdin), Some(0));
        assert_eq!(cache.get_path_id(&imported), Some(1));
        assert!(cache.get_path_id(&missing).is_none());
    }

    #[test]
    fn test_get_path_id_matches_canonical_path() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("lib")).unwrap();
        let imported = source_file(dir.path(), "lib/a.ab");
        // The same file path written differently
        let same_file = dir.path().join("lib/../lib/a.ab");
        let same_file = SourcePath::file(same_file.to_string_lossy()).unwrap();
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &imported);

        assert_eq!(cache.get_path_id(&same_file), Some(1));
    }

    #[test]
    fn test_get_path_id_stdlib() {
        let text = SourcePath::StdLib(String::from("std/text"));
        let array = SourcePath::StdLib(String::from("std/array"));
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &text);

        assert_eq!(cache.get_path_id(&text), Some(1));
        assert!(cache.get_path_id(&array).is_none());
    }

    #[test]
    fn test_add_import_entry_new() {
        let dir = tempdir().unwrap();
        let imported = source_file(dir.path(), "imported.ab");
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        let result = cache.add_import_entry(&SourcePath::Stdin, &imported);

        assert!(result.is_some());
        assert_eq!(cache.files.len(), 2);
        assert_eq!(cache.import_graph[0], vec![1]);
    }

    #[test]
    fn test_add_import_entry_existing() {
        let dir = tempdir().unwrap();
        let target = source_file(dir.path(), "target.ab");
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &target);

        let result = cache.add_import_entry(&SourcePath::Stdin, &target);
        assert!(result.is_some());
        assert_eq!(cache.files.len(), 2);
    }

    #[test]
    fn test_add_import_entry_cycle() {
        let dir = tempdir().unwrap();
        let first = source_file(dir.path(), "first.ab");
        let second = source_file(dir.path(), "second.ab");
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &first);
        cache.add_import_entry(&first, &second);

        assert!(cache.add_import_entry(&second, &first).is_none());
        // The import cycle is not persisted to the cache
        let second_id = cache.get_path_id(&second).unwrap();
        assert!(cache.import_graph[second_id].is_empty());
    }

    #[test]
    fn test_add_code() {
        let dir = tempdir().unwrap();
        let imported = source_file(dir.path(), "imported.ab");
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &imported);

        assert!(cache.get_code(&imported).is_none());
        cache.add_code(&imported, String::from("pub fun foo() {}"));
        assert_eq!(cache.get_code(&imported), Some("pub fun foo() {}"));
    }

    #[test]
    fn test_get_imports_before_metadata() {
        let dir = tempdir().unwrap();
        let imported = source_file(dir.path(), "imported.ab");
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &imported);

        assert!(cache.get_imports(&imported).is_none());
    }

    #[test]
    fn test_topological_sort() {
        let dir = tempdir().unwrap();
        let dep1 = source_file(dir.path(), "dep1.ab");
        let dep2 = source_file(dir.path(), "dep2.ab");
        let mut cache = ImportCache::new(SourcePath::Stdin, String::new());
        cache.add_import_entry(&SourcePath::Stdin, &dep1);
        cache.add_import_entry(&SourcePath::Stdin, &dep2);

        let sorted = cache.topological_sort();
        assert!(!sorted.is_empty());
        assert_eq!(sorted.len(), cache.files.len());
    }
}
