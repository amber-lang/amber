
use super::memory::FunctionDecl;

#[derive(Clone, Debug)]
pub enum ExportUnit {
    Function(FunctionDecl)
}

#[derive(Clone, Debug)]
pub struct Exports {
    values: Vec<ExportUnit>
}

impl Exports {
    pub fn new() -> Exports {
        Exports {
            values: vec![]
        }
    }

    pub fn get_exports(&self) -> &Vec<ExportUnit> {
        &self.values
    }

    pub fn add_function(&mut self, function: FunctionDecl) {
        if function.is_public {
            self.values.push(ExportUnit::Function(function))
        }
    }
}
