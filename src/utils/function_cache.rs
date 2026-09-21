use super::context::Context;
use crate::modules::block::Block;
use crate::modules::function::core::signature::{
    FunctionDeclId, FunctionVariant, FunctionVariantId,
};
use std::collections::HashMap;

#[derive(Debug)]
/// Everything the compiler remembers about one declared function.
pub struct FunctionCacheEntry {
    /// The monomorphic variants produced by call sites so far
    pub variants: Vec<FunctionVariant>,
    /// The context that preserves the function's scope
    pub context: Context,
    /// The pristine declaration body, used as the template for each new variant
    pub block: Block,
    /// Whether the first-pass typecheck with declared types has already been done
    pub first_pass_done: bool,
}

#[derive(Debug, Default)]
// This is a map of all generated functions based on their calls
pub struct FunctionCache {
    pub funs: HashMap<FunctionDeclId, FunctionCacheEntry>,
}

impl FunctionCache {
    pub fn new() -> FunctionCache {
        FunctionCache::default()
    }

    /// Adds a new function declaration to the cache
    pub fn add_declaration(&mut self, id: FunctionDeclId, context: Context, block: Block) {
        self.funs.insert(
            id,
            FunctionCacheEntry {
                variants: Vec::new(),
                context,
                block,
                first_pass_done: false,
            },
        );
    }

    /// Adds a new monomorphized function variant to the cache.
    pub fn add_variant(
        &mut self,
        id: FunctionDeclId,
        mut fun: FunctionVariant,
    ) -> Option<FunctionVariantId> {
        let entry = self.funs.get_mut(&id)?;
        let variant = FunctionVariantId::new(entry.variants.len());
        fun.id = variant;
        entry.variants.push(fun);
        Some(variant)
    }

    /// Gets all the function variants of a function declaration
    pub fn get_variants_cloned(&self, id: FunctionDeclId) -> Option<Vec<FunctionVariant>> {
        self.funs.get(&id).map(|f| f.variants.clone())
    }

    /// Gets all the function variants of a function declaration as a reference
    pub fn get_variants(&self, id: FunctionDeclId) -> Option<&Vec<FunctionVariant>> {
        self.funs.get(&id).map(|f| &f.variants)
    }

    /// Gets function declaration's context
    pub fn get_context(&self, id: FunctionDeclId) -> Option<&Context> {
        self.funs.get(&id).map(|f| &f.context)
    }

    /// Gets the block of a function declaration
    pub fn get_block(&self, id: FunctionDeclId) -> Option<&Block> {
        self.funs.get(&id).map(|f| &f.block)
    }

    /// Checks if the first-pass typecheck has been done for a function
    pub fn is_first_pass_done(&self, id: FunctionDeclId) -> bool {
        self.funs.get(&id).is_some_and(|f| f.first_pass_done)
    }

    /// Marks the first-pass typecheck as done for a function
    pub fn set_first_pass_done(&mut self, id: FunctionDeclId) {
        if let Some(entry) = self.funs.get_mut(&id) {
            entry.first_pass_done = true;
        }
    }
}
