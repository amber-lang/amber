use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::modules::{types::Type, block::Block};
use super::context::Context;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// This is a compiled function instance
pub struct FunctionInstance {
    pub variant_id: usize,
    pub args: Vec<Type>,
    pub returns: Type,
    pub block: Block
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// This is a cached data representing a function
pub struct FunctionCacheEntry {
    pub instances: Vec<FunctionInstance>,
    pub context: Context
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// This is a map of all generated functions based on their invocations
pub struct FunctionCache {
    pub funs: HashMap<usize, FunctionCacheEntry>,
}

impl FunctionCache {
    pub fn new() -> FunctionCache {
        FunctionCache {
            funs: HashMap::new(),
        }
    }

    /// Adds a new function declaration to the cache
    pub fn add_declaration(&mut self, id: usize, context: Context) {
        self.funs.insert(id, FunctionCacheEntry {
            instances: Vec::new(),
            context
        });
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
}