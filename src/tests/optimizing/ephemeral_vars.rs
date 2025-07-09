use crate::{raw_fragment, bash_code, unwrap_fragment};
use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::optimizer::ephemeral_vars::remove_ephemeral_variables;

#[test]
fn test_remove_ephemeral_variables_simple() {
    let mut ast = BlockFragment::new(bash_code!({
        <ephemeral> a = "some value";
        b = a;
        b;
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "b");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarExpr).get_name(), "b");
}

#[test]
fn test_remove_ephemeral_variables_transitive() {
    let mut ast = BlockFragment::new(bash_code!({
        <ephemeral> a = "some value";
        <ephemeral> b = a;
        c = b;
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    dbg!(&ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    let variable = unwrap_fragment!(block.statements[0].clone(), VarStmt);
    assert_eq!(variable.get_name(), "c");
    let expr = unwrap_fragment!(*variable.value, Raw);
    assert_eq!(expr.value, "\"some value\"");
}
