use super::context::Context;
use crate::modules::{block::Block, types::Type};
use std::collections::HashMap;

#[derive(Clone, Debug)]
/// This is a compiled function instance
pub struct FunctionInstance {
    pub variant_id: usize,
    pub args: Vec<Type>,
    pub args_global_ids: Vec<Option<usize>>,
    pub returns: Type,
    pub block: Block,
}

#[derive(Debug)]
/// This is a cached data representing a function
pub struct FunctionCacheEntry {
    pub instances: Vec<FunctionInstance>,
    pub context: Context,
    pub block: Block,
}

#[derive(Debug, Default)]
// This is a map of all generated functions based on their invocations
pub struct FunctionCache {
    pub funs: HashMap<usize, FunctionCacheEntry>,
}

impl FunctionCache {
    pub fn new() -> FunctionCache {
        FunctionCache::default()
    }

    /// Adds a new function declaration to the cache
    pub fn add_declaration(&mut self, id: usize, context: Context, block: Block) {
        self.funs.insert(
            id,
            FunctionCacheEntry {
                instances: Vec::new(),
                context,
                block,
            },
        );
    }

    /// Adds a new function instance to the cache
    pub fn add_instance(&mut self, id: usize, mut fun: FunctionInstance) -> usize {
        let functions = self.funs.get_mut(&id).expect("Function not found in cache");
        fun.variant_id = functions.instances.len();
        functions.instances.push(fun);
        functions.instances.len() - 1
    }

    /// Gets all the function instances of a function declaration
    pub fn get_instances_cloned(&self, id: usize) -> Option<Vec<FunctionInstance>> {
        self.funs.get(&id).map(|f| f.instances.clone())
    }

    /// Gets all the function instances of a function declaration as a reference
    pub fn get_instances(&self, id: usize) -> Option<&Vec<FunctionInstance>> {
        self.funs.get(&id).map(|f| &f.instances)
    }

    /// Gets the context of a function declaration
    pub fn get_context(&self, id: usize) -> Option<&Context> {
        self.funs.get(&id).map(|f| &f.context)
    }

    /// Gets the block of a function declaration
    pub fn get_block(&self, id: usize) -> Option<&Block> {
        self.funs.get(&id).map(|f| &f.block)
    }
}
