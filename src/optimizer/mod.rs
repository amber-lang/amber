use crate::modules::prelude::*;
use ephemeral_vars::remove_ephemeral_variables;
use unused_vars::remove_unused_variables;

pub mod ephemeral_vars;
pub mod unused_vars;

pub fn optimize_fragments(ast: &mut FragmentKind) {
    remove_unused_variables(ast);
    remove_ephemeral_variables(ast);
}
