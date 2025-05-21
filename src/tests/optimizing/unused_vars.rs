use crate::translate::fragments::var_expr::VarIndexValue;
use crate::raw_fragment;
use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::optimizer::unused_var::remove_unused_variables;

macro_rules! unwrap_fragment {
    ($expr:expr, $kind:ident) => {{
        match $expr {
            FragmentKind::$kind(fragment) => fragment,
            _ => panic!("Expected FragmentKind::{}", stringify!($kind)),
        }
    }};
}

#[test]
fn test_remove_unused_variables_simple() {
    let a_stmt = VarStmtFragment::new("a", Type::Num, raw_fragment!("some value"));
    let b_stmt = VarStmtFragment::new("b", Type::Num, raw_fragment!("another value"));
    let a_expr = VarExprFragment::from_stmt(&a_stmt);

    let mut ast = BlockFragment::new(vec![
        a_stmt.to_frag(),
        b_stmt.to_frag(),
        a_expr.to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarExpr).get_name(), "a");
}

#[test]
fn test_remove_unused_variables_nested_blocks() {
    let outer_a_stmt = VarStmtFragment::new("outer_a", Type::Num, raw_fragment!("10"));
    let outer_b_stmt = VarStmtFragment::new("outer_b", Type::Num, raw_fragment!("20"));
    let inner_a_stmt = VarStmtFragment::new("inner_a", Type::Num, raw_fragment!("30"));
    let inner_b_stmt = VarStmtFragment::new("inner_b", Type::Num, raw_fragment!("40"));

    let outer_a_expr = VarExprFragment::from_stmt(&outer_a_stmt);
    let inner_a_expr = VarExprFragment::from_stmt(&inner_a_stmt);

    // Create outer block containing inner block
    let mut ast = BlockFragment::new(vec![
        outer_a_stmt.to_frag(),
        outer_b_stmt.to_frag(),
        BlockFragment::new(vec![
            inner_a_stmt.to_frag(),
            inner_b_stmt.to_frag(),
            inner_a_expr.to_frag(),
            outer_a_expr.clone().to_frag(),
        ], true).to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    // Check that outer block retains outer_a but not outer_b
    let outer_block = unwrap_fragment!(ast, Block);
    assert_eq!(outer_block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(outer_block.statements[0].clone(), VarStmt).get_name(), "outer_a");

    // Check that inner block retains inner_a but not inner_b
    let inner_block = unwrap_fragment!(outer_block.statements[1].clone(), Block);
    assert_eq!(inner_block.statements.len(), 3);
    assert_eq!(unwrap_fragment!(inner_block.statements[0].clone(), VarStmt).get_name(), "inner_a");
    assert_eq!(unwrap_fragment!(inner_block.statements[1].clone(), VarExpr).get_name(), "inner_a");
    assert_eq!(unwrap_fragment!(inner_block.statements[2].clone(), VarExpr).get_name(), "outer_a");
}

#[test]
fn test_remove_unused_variables_interpolable() {
    let a_stmt = VarStmtFragment::new("a", Type::Num, raw_fragment!("10"));
    let b_stmt = VarStmtFragment::new("b", Type::Num, raw_fragment!("20"));
    let c_stmt = VarStmtFragment::new("c", Type::Num, raw_fragment!("30"));

    let b_expr = VarExprFragment::from_stmt(&b_stmt);
    let interp = InterpolableFragment::new(
        vec!["text with ".to_string(), " interpolated".to_string()],
        vec![b_expr.to_frag()],
        InterpolableRenderType::StringLiteral
    );

    let mut ast = BlockFragment::new(vec![
        a_stmt.to_frag(),
        b_stmt.to_frag(),
        c_stmt.to_frag(),
        interp.to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "b");
    assert!(matches!(block.statements[1].clone(), FragmentKind::Interpolable(_)));
}

#[test]
fn test_remove_unused_variables_subprocess() {
    let a_stmt = VarStmtFragment::new("a", Type::Num, raw_fragment!("10"));
    let b_stmt = VarStmtFragment::new("b", Type::Num, raw_fragment!("20"));

    let a_expr = VarExprFragment::from_stmt(&a_stmt);
    let subprocess = SubprocessFragment::new(a_expr.to_frag());

    let mut ast = BlockFragment::new(vec![
        a_stmt.to_frag(),
        b_stmt.to_frag(),
        subprocess.to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert!(matches!(block.statements[1].clone(), FragmentKind::Subprocess(_)));
}

#[test]
fn test_remove_unused_variables_transitive() {
    let a_stmt = VarStmtFragment::new("a", Type::Num, raw_fragment!("10"));
    let a_expr = VarExprFragment::from_stmt(&a_stmt);

    let b_stmt = VarStmtFragment::new("b", Type::Num, a_expr.to_frag());
    let b_expr = VarExprFragment::from_stmt(&b_stmt);

    let c_stmt = VarStmtFragment::new("c", Type::Num, b_expr.to_frag());

    let mut ast = BlockFragment::new(vec![
        a_stmt.to_frag(),
        b_stmt.to_frag(),
        c_stmt.to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 0);
}

#[test]
fn test_remove_unused_variables_in_list() {
    let a_stmt = VarStmtFragment::new("a", Type::Num, raw_fragment!("10"));
    let b_stmt = VarStmtFragment::new("b", Type::Num, raw_fragment!("20"));
    let c_stmt = VarStmtFragment::new("c", Type::Num, raw_fragment!("30"));

    let a_expr = VarExprFragment::from_stmt(&a_stmt);
    let b_expr = VarExprFragment::from_stmt(&b_stmt);

    let mut ast = BlockFragment::new(vec![
        a_stmt.to_frag(),
        b_stmt.to_frag(),
        c_stmt.to_frag(),
        ListFragment::new(vec![
            a_expr.to_frag(),
            b_expr.to_frag(),
        ]).with_spaces().to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "b");
    assert!(matches!(block.statements[2].clone(), FragmentKind::List(_)));
}

#[test]
fn test_remove_unused_variables_array_indexing() {
    let array_stmt = VarStmtFragment::new("array", Type::array_of(Type::Num), raw_fragment!("1 2 3 4 5"));
    let index_stmt = VarStmtFragment::new("index", Type::Num, raw_fragment!("2"));
    let unused_stmt = VarStmtFragment::new("unused", Type::Num, raw_fragment!("0"));

    let index_expr = VarExprFragment::from_stmt(&index_stmt);
    let mut array_expr = VarExprFragment::from_stmt(&array_stmt);

    // TODO: Replece this with `with_index_by_value` when #702 gets merged
    array_expr.index = Some(Box::new(VarIndexValue::Index(index_expr.to_frag())));

    let mut ast = BlockFragment::new(vec![
        array_stmt.to_frag(),
        index_stmt.to_frag(),
        unused_stmt.to_frag(),
        array_expr.to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "array");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "index");
    assert!(matches!(block.statements[2].clone(), FragmentKind::VarExpr(_)));
}

#[test]
fn test_remove_unused_variables_multiple_references() {
    let a_stmt = VarStmtFragment::new("a", Type::Num, raw_fragment!("10"));
    let b_stmt = VarStmtFragment::new("b", Type::Num, raw_fragment!("20"));

    let a_expr1 = VarExprFragment::from_stmt(&a_stmt);
    let a_expr2 = VarExprFragment::from_stmt(&a_stmt);
    let a_expr3 = VarExprFragment::from_stmt(&a_stmt);

    let mut ast = BlockFragment::new(vec![
        a_stmt.to_frag(),
        b_stmt.to_frag(),
        a_expr1.to_frag(),
        a_expr2.to_frag(),
        a_expr3.to_frag(),
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);

    assert_eq!(block.statements.len(), 4);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarExpr).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[2].clone(), VarExpr).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[3].clone(), VarExpr).get_name(), "a");
}

#[test]
fn test_nested_scope_transitive() {
    let outer_var_stmt = VarStmtFragment::new("outer_var", Type::Num, raw_fragment!("10"));
    let outer_var_expr = VarExprFragment::from_stmt(&outer_var_stmt);
    let outer_unused_stmt = VarStmtFragment::new("outer_unused", Type::Num, raw_fragment!("20"));
    let outer_unused_expr = VarExprFragment::from_stmt(&outer_unused_stmt);

    let inner_var_stmt = VarStmtFragment::new("inner_var", Type::Num, outer_var_expr.to_frag());
    let inner_unused_stmt = VarStmtFragment::new("inner_unused", Type::Num, outer_unused_expr.to_frag());
    let inner_var_expr = VarExprFragment::from_stmt(&inner_var_stmt);

    // Build the complete AST
    let mut ast = BlockFragment::new(vec![
        outer_var_stmt.to_frag(),
        outer_unused_stmt.to_frag(),
        BlockFragment::new(vec![
            inner_var_stmt.to_frag(),
            inner_unused_stmt.to_frag(),
            inner_var_expr.to_frag(),
        ], true).to_frag()
    ], true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "outer_var");
    let inner_block = unwrap_fragment!(block.statements[1].clone(), Block);
    assert_eq!(inner_block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(inner_block.statements[0].clone(), VarStmt).get_name(), "inner_var");
    assert_eq!(unwrap_fragment!(inner_block.statements[1].clone(), VarExpr).get_name(), "inner_var");
}
