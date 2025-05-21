use crate::modules::prelude::*;
use var::remove_unused_variables;

pub mod var;

pub fn optimize_fragments(ast: &mut FragmentKind) {
    remove_unused_variables(ast);
}
