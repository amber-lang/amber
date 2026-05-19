use crate::modules::block::Block;
use crate::modules::types::Type;
use crate::utils::context::Context;
use crate::utils::function_cache::{FunctionCache, FunctionInstance};
use heraclitus_compiler::compiling::SyntaxModule;

#[cfg(test)]
mod function_cache_tests {
    use super::*;

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

        cache.add_declaration(42, ctx, block);

        assert!(cache.get_instances_cloned(42).is_some());
        assert!(cache.get_instances_cloned(99).is_none());
    }

    #[test]
    fn test_function_cache_add_instance() {
        let mut cache = FunctionCache::new();
        let ctx = Context::new(None, vec![]);
        let block = Block::new();

        cache.add_declaration(1, ctx, block);

        let instance = FunctionInstance {
            variant_id: 0,
            args: vec![],
            args_global_ids: vec![],
            returns: Type::Null,
            block: Block::new(),
        };

        let variant_idx = cache.add_instance(1, instance);
        assert_eq!(variant_idx, 0);

        let instances = cache.get_instances_cloned(1).unwrap();
        assert_eq!(instances.len(), 1);
    }

    #[test]
    fn test_function_cache_first_pass() {
        let mut cache = FunctionCache::new();
        let ctx = Context::new(None, vec![]);
        let block = Block::new();

        cache.add_declaration(1, ctx, block);

        assert!(!cache.is_first_pass_done(1));
        cache.set_first_pass_done(1);
        assert!(cache.is_first_pass_done(1));
    }
}
