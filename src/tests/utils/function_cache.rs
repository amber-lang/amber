//! Tests for utils/function_cache.rs

use crate::compiler::SourcePath;
use crate::modules::block::Block;
use crate::modules::function::core::signature::{
    FunctionDeclId, FunctionVariant, FunctionVariantId,
};
use crate::modules::types::Type;
use crate::utils::context::Context;
use crate::utils::function_cache::FunctionCache;
use heraclitus_compiler::compiling::SyntaxModule;

#[cfg(test)]
mod function_cache_tests {
    use super::*;

    /// A cache with a single declared function, and that function's id.
    fn cache_with_declaration() -> (FunctionCache, FunctionDeclId) {
        let mut cache = FunctionCache::new();
        let id = FunctionDeclId::new(1);
        cache.add_declaration(id, Context::new(SourcePath::Stdin, vec![]), Block::new());
        (cache, id)
    }

    /// An empty typechecked variant with the given id.
    fn variant(id: FunctionVariantId) -> FunctionVariant {
        FunctionVariant {
            id,
            param_types: vec![],
            param_global_ids: vec![],
            returns: Type::Null,
            body: Block::new(),
        }
    }

    /// Ids of the variants the cache exposes, in the order it exposes them.
    fn get_variant_ids(cache: &FunctionCache, id: FunctionDeclId) -> Vec<FunctionVariantId> {
        cache.get_variants(id).unwrap().map(|v| v.id).collect()
    }

    #[test]
    fn test_function_cache_declaration_starts_without_variants() {
        let (cache, id) = cache_with_declaration();
        assert_eq!(get_variant_ids(&cache, id), vec![]);
    }

    #[test]
    fn test_function_cache_unknown_declaration() {
        let (mut cache, _) = cache_with_declaration();
        let unknown = FunctionDeclId::new(99);
        assert!(cache.get_variants(unknown).is_none());
        assert_eq!(cache.reserve_variant_id(unknown), None);
        assert_eq!(
            cache.insert_reserved_variant(unknown, variant(FunctionVariantId::new(0))),
            None
        );
    }

    #[test]
    fn test_function_cache_reserves_increasing_variant_ids() {
        let (mut cache, id) = cache_with_declaration();
        assert_eq!(
            cache.reserve_variant_id(id),
            Some(FunctionVariantId::new(0))
        );
        assert_eq!(
            cache.reserve_variant_id(id),
            Some(FunctionVariantId::new(1))
        );
    }

    #[test]
    fn test_function_cache_keeps_reserved_ids_when_finished_out_of_order() {
        let (mut cache, id) = cache_with_declaration();

        // The outer variant reserves first, but a nested variant finishes first
        let outer = cache.reserve_variant_id(id).unwrap();
        let nested = cache.reserve_variant_id(id).unwrap();
        assert_eq!(
            cache.insert_reserved_variant(id, variant(nested)),
            Some(nested)
        );
        assert_eq!(
            cache.insert_reserved_variant(id, variant(outer)),
            Some(outer)
        );

        assert_eq!(get_variant_ids(&cache, id), vec![outer, nested]);
    }

    #[test]
    fn test_function_cache_hides_unfinished_variants() {
        let (mut cache, id) = cache_with_declaration();

        // A reserved variant that never finishes typechecking is not exposed
        let _unfinished = cache.reserve_variant_id(id).unwrap();
        let finished = cache.reserve_variant_id(id).unwrap();
        cache.insert_reserved_variant(id, variant(finished));

        assert_eq!(get_variant_ids(&cache, id), vec![finished]);
        assert_eq!(cache.get_variants_cloned(id).unwrap().len(), 1);
    }

    #[test]
    #[should_panic(expected = "Function variant id was never reserved")]
    fn test_function_cache_rejects_unreserved_variant_id() {
        let (mut cache, id) = cache_with_declaration();
        cache.insert_reserved_variant(id, variant(FunctionVariantId::new(0)));
    }

    #[test]
    fn test_function_cache_first_pass() {
        let (mut cache, id) = cache_with_declaration();
        assert!(!cache.is_first_pass_done(id));
        cache.set_first_pass_done(id);
        assert!(cache.is_first_pass_done(id));
    }
}
