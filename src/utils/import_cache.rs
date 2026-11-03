use crate::compiler::SourcePath;
use crate::modules::block::Block;
use crate::modules::function::core::signature::FunctionSignature;
use crate::utils::context::VariableDecl;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub block: Block,
    pub pub_vars: BTreeMap<String, VariableDecl>,
    pub pub_funs: BTreeMap<String, FunctionSignature>,
}

#[derive(Debug, Clone)]
pub struct FileCache {
    pub path: SourcePath,
    pub code: Option<String>,
    pub metadata: Option<FileMetadata>,
}

#[derive(Debug, Clone)]
pub struct ImportCache {
    /// The paths of the imports (used to be able to resolve imports with topological sort)
    pub import_graph: Vec<Vec<usize>>,
    /// Cached imported files (always has the same length as the import graph)
    pub files: Vec<FileCache>,
}

impl ImportCache {
    pub fn get_path_id(&self, path: &SourcePath) -> Option<usize> {
        self.files.iter().position(|candidate| candidate.path == *path)
    }

    pub fn new(initial_path: SourcePath, initial_code: String) -> Self {
        ImportCache {
            files: vec![FileCache {
                path: initial_path,
                code: Some(initial_code),
                metadata: None,
            }],
            import_graph: vec![vec![]],
        }
    }

    fn contains_cycle_util(
        &self,
        v: usize,
        visited: &mut Vec<bool>,
        rec_stack: &mut Vec<bool>,
    ) -> bool {
        if !visited[v] {
            visited[v] = true;
            rec_stack[v] = true;
            for i in self.import_graph[v].iter() {
                if (!visited[*i] && self.contains_cycle_util(*i, visited, rec_stack))
                    || rec_stack[*i]
                {
                    return true;
                }
            }
        }
        rec_stack[v] = false;
        false
    }

    /// Check if graph contains a cycle starting from id
    pub fn contains_cycle(&self, id: usize) -> bool {
        let mut visited = vec![false; self.files.len()];
        let mut stack = vec![false; self.files.len()];
        self.contains_cycle_util(id, &mut visited, &mut stack)
    }

    /// Registers that `src_path` imports `dst_path`
    /// Returns the id of `dst_path` or `None` if the import creates a cycle
    pub fn add_import_entry(&mut self, src_path: &SourcePath, dst_path: &SourcePath) -> Option<usize> {
        let src_path_id = self
            .get_path_id(src_path)
            .expect("Importing file must already be in the import cache");
        let dst_path_id = match self.get_path_id(dst_path) {
            Some(dst_path_id) => dst_path_id,
            None => {
                self.files.push(FileCache {
                    path: dst_path.clone(),
                    code: None,
                    metadata: None,
                });
                self.import_graph.push(vec![]);
                self.files.len() - 1
            }
        };
        if !self.import_graph[src_path_id].contains(&dst_path_id) {
            self.import_graph[src_path_id].push(dst_path_id);
        }
        // Do not leave the cyclic edge in the graph
        if self.contains_cycle(src_path_id) {
            self.import_graph[src_path_id].retain(|&id| id != dst_path_id);
            return None;
        }
        Some(dst_path_id)
    }

    pub fn add_import_metadata(
        &mut self,
        path: &SourcePath,
        block: Block,
        pub_vars: BTreeMap<String, VariableDecl>,
        pub_funs: BTreeMap<String, FunctionSignature>,
    ) {
        let path_id = self
            .get_path_id(path)
            .expect("Imported file must already be in the import cache");
        self.files[path_id].metadata = Some(FileMetadata {
            block,
            pub_vars,
            pub_funs,
        });
    }

    pub fn add_code(&mut self, path: &SourcePath, code: String) {
        let path_id = self
            .get_path_id(path)
            .expect("Imported file must already be in the import cache");
        self.files[path_id].code = Some(code);
    }

    pub fn get_imports(
        &self,
        path: &SourcePath,
    ) -> Option<(
        BTreeMap<String, VariableDecl>,
        BTreeMap<String, FunctionSignature>
    )> {
        let path_id = self.get_path_id(path)?;
        self.files[path_id]
            .metadata
            .as_ref()
            .map(|meta| (meta.pub_vars.clone(), meta.pub_funs.clone()))
    }

    pub fn get_code(&self, path: &SourcePath) -> Option<&str> {
        let id = self.get_path_id(path)?;
        self.files[id].code.as_deref()
    }

    fn topological_sort_util(&self, v: usize, visited: &mut Vec<bool>, stack: &mut Vec<usize>) {
        visited[v] = true;
        for i in self.import_graph[v].iter() {
            if !visited[*i] {
                self.topological_sort_util(*i, visited, stack);
            }
        }
        stack.push(v);
    }

    pub fn topological_sort(&self) -> Vec<usize> {
        let mut stack = Vec::new();
        let mut visited = vec![false; self.files.len()];
        self.topological_sort_util(0, &mut visited, &mut stack);
        stack
    }
}
