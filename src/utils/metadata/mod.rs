mod parser;
mod translate;
pub use parser::*;
pub use translate::*;

#[derive(Debug)]
pub struct TargetShell {
    pub shell: ShellType,
}
