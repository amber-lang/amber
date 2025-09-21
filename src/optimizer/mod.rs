use crate::modules::prelude::*;
use unused_vars::remove_unused_variables;
use ephemeral_vars::remove_ephemeral_variables;

pub mod ephemeral_vars;
pub mod unused_vars;

pub fn optimize_fragments(ast: &mut FragmentKind) {
    remove_unused_variables(ast);
    remove_ephemeral_variables(ast);
}
