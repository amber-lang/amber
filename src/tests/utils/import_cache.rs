//! Tests for utils/import_cache.rs

use crate::utils::import_cache::ImportCache;

#[cfg(test)]
mod import_cache_tests {
    use super::*;

    #[test]
    fn test_import_cache_new() {
        let cache = ImportCache::new(None);
        assert_eq!(cache.files.len(), 1);
        assert_eq!(cache.files[0].path, ".");
        assert!(cache.import_graph.len() == 1);
    }

    #[test]
    fn test_import_cache_new_with_path() {
        let cache = ImportCache::new(Some("/test/path.ab".to_string()));
        assert_eq!(cache.files[0].path, "/test/path.ab");
    }

    #[test]
    fn test_get_path_id() {
        let mut cache = ImportCache::new(None);
        cache.add_import_entry(None, "/new/path.ab".to_string());
        
        assert_eq!(cache.get_path_id("."), Some(0));
        assert_eq!(cache.get_path_id("/new/path.ab"), Some(1));
        assert!(cache.get_path_id("/nonexistent.ab").is_none());
    }

    #[test]
    fn test_add_import_entry_new() {
        let mut cache = ImportCache::new(None);
        let result = cache.add_import_entry(None, "/imported.ab".to_string());
        
        assert!(result.is_some());
        assert_eq!(cache.files.len(), 2);
        assert_eq!(cache.import_graph[0], vec![1]);
    }

    #[test]
    fn test_add_import_entry_existing() {
        let mut cache = ImportCache::new(None);
        cache.add_import_entry(None, "/target.ab".to_string());
        
        let result = cache.add_import_entry(None, "/target.ab".to_string());
        assert!(result.is_some());
        assert_eq!(cache.files.len(), 2);
    }

    #[test]
    fn test_topological_sort() {
        let mut cache = ImportCache::new(None);
        cache.add_import_entry(None, "/dep1.ab".to_string());
        cache.add_import_entry(None, "/dep2.ab".to_string());
        
        let sorted = cache.topological_sort();
        assert!(!sorted.is_empty());
        assert_eq!(sorted.len(), cache.files.len());
    }
}
