use crate::modules::prelude::*;
use unused_var::remove_unused_variables;

pub mod unused_var;

pub fn optimize_fragments(ast: &mut FragmentKind) {
    remove_unused_variables(ast);
}
