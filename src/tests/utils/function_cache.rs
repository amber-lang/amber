use crate::modules::block::Block;
use crate::modules::function::core::signature::{FunctionVariant, FunctionVariantId};
use crate::modules::types::Type;
use crate::utils::context::Context;
use crate::utils::function_cache::FunctionCache;
use heraclitus_compiler::compiling::SyntaxModule;

#[cfg(test)]
mod function_cache_tests {
    use crate::modules::function::core::signature::FunctionDeclId;

    use super::*;

    fn monomorph() -> FunctionVariant {
        FunctionVariant {
            id: FunctionVariantId::new(0),
            param_types: vec![],
            param_global_ids: vec![],
            returns: Type::Null,
            body: Block::new(),
        }
    }

    #[test]
    fn test_function_cache_new() {
        let cache = FunctionCache::new();
        assert!(cache.funs.is_empty());
    }

    #[test]
    fn test_function_cache_add_declaration() {
        let mut cache = FunctionCache::new();
        let ctx = Context::new(None, vec![]);
        let block = Block::new();

        cache.add_declaration(FunctionDeclId::new(42), ctx, block);

        assert!(cache.get_variants_cloned(FunctionDeclId::new(42)).is_some());
        assert!(cache.get_variants_cloned(FunctionDeclId::new(99)).is_none());
    }

    #[test]
    fn test_function_cache_add_variant() {
        let mut cache = FunctionCache::new();
        let ctx = Context::new(None, vec![]);
        let block = Block::new();

        cache.add_declaration(FunctionDeclId::new(1), ctx, block);

        let variant = cache.add_variant(FunctionDeclId::new(1), monomorph());
        assert_eq!(variant, Some(FunctionVariantId::new(0)));

        let variants = cache.get_variants_cloned(FunctionDeclId::new(1)).unwrap();
        assert_eq!(variants.len(), 1);
    }

    #[test]
    fn test_function_cache_add_variant_without_declaration() {
        let mut cache = FunctionCache::new();
        assert_eq!(cache.add_variant(FunctionDeclId::new(7), monomorph()), None);
    }

    #[test]
    fn test_function_cache_assigns_increasing_variant_ids() {
        let mut cache = FunctionCache::new();
        cache.add_declaration(
            FunctionDeclId::new(1),
            Context::new(None, vec![]),
            Block::new(),
        );

        assert_eq!(
            cache.add_variant(FunctionDeclId::new(1), monomorph()),
            Some(FunctionVariantId::new(0))
        );
        assert_eq!(
            cache.add_variant(FunctionDeclId::new(1), monomorph()),
            Some(FunctionVariantId::new(1))
        );
    }

    #[test]
    fn test_function_cache_first_pass() {
        let mut cache = FunctionCache::new();
        let ctx = Context::new(None, vec![]);
        let block = Block::new();

        cache.add_declaration(FunctionDeclId::new(1), ctx, block);

        assert!(!cache.is_first_pass_done(FunctionDeclId::new(1)));
        cache.set_first_pass_done(FunctionDeclId::new(1));
        assert!(cache.is_first_pass_done(FunctionDeclId::new(1)));
    }
}
