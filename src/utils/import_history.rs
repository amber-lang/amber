use crate::modules::block::Block;

use super::exports::Exports;

#[derive(Debug, Clone)]
pub struct ImportHistory {
    pub imports: Vec<String>,
    pub import_graph: Vec<Vec<usize>>,
    // Used to resolve imports in the correct order (topological sort)
    pub import_blocks: Vec<Option<Block>>,
    // Used to resolve problem with 
    pub exports: Vec<Option<Exports>>
}

impl ImportHistory {

    pub fn get_path(optional_path: Option<String>) -> String {
        optional_path.unwrap_or_else(|| String::from("."))
    }

    pub fn get_path_id(&self, path: &str) -> Option<usize> {
        self.imports.iter().position(|import| import == path)
    }

    pub fn new(initial_path: Option<String>) -> Self {
        ImportHistory {
            imports: vec![Self::get_path(initial_path)],
            import_graph: vec![vec![]],
            import_blocks: vec![None],
            exports: vec![None]
        }
    }

    fn contains_cycle_util(&self, v: usize, visited: &mut Vec<bool>, rec_stack: &mut Vec<bool>) -> bool {
        if !visited[v] {
            visited[v] = true;
            rec_stack[v] = true;
            for i in self.import_graph[v].iter() {
                if (!visited[*i] && self.contains_cycle_util(*i, visited, rec_stack)) || rec_stack[*i] {
                    return true;
                }
            }
        }
        rec_stack[v] = false;
        false
    }

    // Check if graph contains a cycle starting from id
    pub fn contains_cycle(&self, id: usize) -> bool {
        let mut visited = vec![false; self.imports.len()];
        let mut stack = vec![false; self.imports.len()];
        self.contains_cycle_util(id, &mut visited, &mut stack)
    }

    pub fn add_import(&mut self, src: Option<String>, path: String) -> Option<usize> {
        let src_id = self.get_path_id(&Self::get_path(src)).unwrap();
        match self.get_path_id(&path) {
            Some(dst_id) => {
                self.import_graph[src_id].push(dst_id);
                if self.contains_cycle(src_id) {
                    None
                } else {
                    Some(dst_id)
                }
            }
            None => {
                let dst_id = self.imports.len();
                self.imports.push(path);
                self.exports.push(None);
                self.import_blocks.push(None);
                self.import_graph.push(vec![]);
                self.import_graph[src_id].push(dst_id);
                Some(dst_id)
            }
        }
    }

    pub fn add_import_block(&mut self, path: Option<String>, block: Block) {
        let path_id = self.get_path_id(&Self::get_path(path)).unwrap();
        self.import_blocks[path_id] = Some(block);
    }

    pub fn add_export(&mut self, path: Option<String>, exports: Exports) {
        let path_id = self.get_path_id(&Self::get_path(path)).unwrap();
        self.exports[path_id] = Some(exports);
    }

    pub fn get_export(&mut self, path: Option<String>) -> Option<Exports> {
        if let Some(path_id) = self.get_path_id(&Self::get_path(path)) {
            self.exports[path_id].clone()
        } else {
            None
        }
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
        let mut visited = vec![false; self.imports.len()];
        self.topological_sort_util(0, &mut visited, &mut stack);
        stack
    }
}