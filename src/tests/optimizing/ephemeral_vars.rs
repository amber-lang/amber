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
    assert_eq!(block.statements.len(), 1);
    let variable = unwrap_fragment!(block.statements[0].clone(), VarStmt);
    assert_eq!(variable.get_name(), "c");
    let expr = unwrap_fragment!(*variable.value, Raw);
    assert_eq!(expr.value, "\"some value\"");
}

#[test]
fn test_remove_ephemeral_variables_no_optimization_non_ephemeral() {
    let mut ast = BlockFragment::new(bash_code!({
        a = "some value";
        b = a;
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "b");
}

#[test]
fn test_remove_ephemeral_variables_no_optimization_different_names() {
    let mut ast = BlockFragment::new(bash_code!({
        <ephemeral> a = "some value";
        b = c;
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "b");
}

#[test]
fn test_remove_ephemeral_variables_multiple_separate_chains() {
    let mut ast = BlockFragment::new(bash_code!({
        <ephemeral> a = "value1";
        b = a;
        <ephemeral> c = "value2";
        d = c;
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);

    let first_var = unwrap_fragment!(block.statements[0].clone(), VarStmt);
    assert_eq!(first_var.get_name(), "b");
    let first_expr = unwrap_fragment!(*first_var.value, Raw);
    assert_eq!(first_expr.value, "\"value1\"");

    let second_var = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(second_var.get_name(), "d");
    let second_expr = unwrap_fragment!(*second_var.value, Raw);
    assert_eq!(second_expr.value, "\"value2\"");
}

#[test]
fn test_remove_ephemeral_variables_long_transitive_chain() {
    let mut ast = BlockFragment::new(bash_code!({
        <ephemeral> a = "original value";
        <ephemeral> b = a;
        <ephemeral> c = b;
        <ephemeral> d = c;
        final_var = d;
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 1);

    let variable = unwrap_fragment!(block.statements[0].clone(), VarStmt);
    assert_eq!(variable.get_name(), "final_var");
    let expr = unwrap_fragment!(*variable.value, Raw);
    assert_eq!(expr.value, "\"original value\"");
}

#[test]
fn test_remove_ephemeral_variables_single_ephemeral() {
    let mut ast = BlockFragment::new(bash_code!({
        <ephemeral> a = "unused value";
        b = "other value";
    }), true).to_frag();

    remove_ephemeral_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "b");
}
