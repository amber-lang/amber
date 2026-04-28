use super::context::FunctionDecl;
use crate::{modules::block::Block, utils::context::VariableDecl};

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub block: Block,
    pub pub_funs: Vec<FunctionDecl>,
    pub pub_vars: Vec<VariableDecl>,
}

#[derive(Debug, Clone)]
pub struct FileCache {
    pub path: String,
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
    pub fn get_path(optional_path: Option<String>) -> String {
        optional_path.unwrap_or_else(|| String::from("."))
    }

    pub fn get_path_id(&self, path: &str) -> Option<usize> {
        self.files.iter().position(|import| import.path == path)
    }

    pub fn new(initial_path: Option<String>) -> Self {
        ImportCache {
            files: vec![FileCache {
                path: Self::get_path(initial_path),
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

    // Check if graph contains a cycle starting from id
    pub fn contains_cycle(&self, id: usize) -> bool {
        let mut visited = vec![false; self.files.len()];
        let mut stack = vec![false; self.files.len()];
        self.contains_cycle_util(id, &mut visited, &mut stack)
    }

    pub fn add_import_entry(
        &mut self,
        src_path: Option<String>,
        dst_path: String,
    ) -> Option<usize> {
        // Get id of source path
        let src_path_id = self.get_path_id(&Self::get_path(src_path)).unwrap();
        // Check if destination path is already in the graph
        match self.get_path_id(&dst_path) {
            // If so add it to the graph
            Some(dst_path_id) => {
                self.import_graph[src_path_id].push(dst_path_id);
                (!self.contains_cycle(src_path_id)).then_some(dst_path_id)
            }
            // If not add it to the graph and create a new import entry
            None => {
                let dst_path_id = self.files.len();
                self.files.push(FileCache {
                    path: dst_path,
                    metadata: None,
                });
                self.import_graph.push(vec![]);
                self.import_graph[src_path_id].push(dst_path_id);
                Some(dst_path_id)
            }
        }
    }

    pub fn add_import_metadata(
        &mut self,
        path: Option<String>,
        block: Block,
        pub_funs: Vec<FunctionDecl>,
        pub_vars: Vec<VariableDecl>,
    ) {
        let path_id = self.get_path_id(&Self::get_path(path)).unwrap();
        self.files[path_id].metadata = Some(FileMetadata {
            block,
            pub_funs,
            pub_vars,
        });
    }

    pub fn get_imports(
        &mut self,
        path: Option<String>,
    ) -> Option<(Vec<FunctionDecl>, Vec<VariableDecl>)> {
        self.get_path_id(&Self::get_path(path)).and_then(|path_id| {
            self.files[path_id]
                .metadata
                .as_ref()
                .map(|meta| (meta.pub_funs.clone(), meta.pub_vars.clone()))
        })
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
