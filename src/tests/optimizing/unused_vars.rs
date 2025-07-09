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

macro_rules! bash_code {
    // Base case
    (@acc [$($elems:expr),*]) => {
        vec![$($elems),*]
    };
    // Variable assignments
    (@acc [$($elems:expr),*] $a:ident = $b:ident; $($rest:tt)*) => {{
        let value = VarExprFragment::new(stringify!($b), Type::Generic).to_frag();
        let variable = VarStmtFragment::new(stringify!($a), Type::Generic, value).to_frag();
        bash_code!(@acc [$($elems,)* variable] $($rest)*)
    }};
    (@acc [$($elems:expr),*] $a:ident = $b:literal; $($rest:tt)*) => {{
        let variable = VarStmtFragment::new(stringify!($a), Type::Generic, raw_fragment!(stringify!($b))).to_frag();
        bash_code!(@acc [$($elems,)* variable] $($rest)*)
    }};
    // Blocks
    (@acc [$($elems:expr),*] if { $($cond_block:tt)* } $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* BlockFragment::new(bash_code!({ $($cond_block)* }), true).with_condition(true).to_frag()] $($rest)*)
    };
    (@acc [$($elems:expr),*] { $($cond_block:tt)* } $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* BlockFragment::new(bash_code!({ $($cond_block)* }), true).to_frag()] $($rest)*)
    };
    // Other syntax
    (@acc [$($elems:expr),*] syntax($expr:expr); $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* $expr] $($rest)*)
    };
    // Variable expression
    (@acc [$($elems:expr),*] $var:ident; $($rest:tt)*) => {
        bash_code!(@acc [$($elems,)* VarExprFragment::new(stringify!($var), Type::Generic).to_frag()] $($rest)*)
    };
    ({ $($tokens:tt)* }) => {
        bash_code!(@acc [] $($tokens)*)
    };
}

#[test]
fn test_remove_unused_variables_simple() {
    let mut ast = BlockFragment::new(bash_code!({
        a = "some value";
        b = "another value";
        a;
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarExpr).get_name(), "a");
}

#[test]
fn test_remove_unused_variables_nested_blocks() {
    // Create outer block containing inner block
    let mut ast = BlockFragment::new(bash_code!({
        outer_a = 10;
        outer_b = 20;
        {
            inner_a = 30;
            inner_b = 40;
            inner_a;
            outer_a;
        }
    }), true).to_frag();

    dbg!(ast.clone());

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
    let interp = InterpolableFragment::new(
        vec!["text with ".to_string(), " interpolated".to_string()],
        vec![VarExprFragment::new("b", Type::Generic).to_frag()],
        InterpolableRenderType::StringLiteral
    );

    let mut ast = BlockFragment::new(bash_code!({
        a = 10;
        b = 20;
        c = 30;
        syntax(interp.to_frag());
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "b");
    assert!(matches!(block.statements[1].clone(), FragmentKind::Interpolable(_)));
}

#[test]
fn test_remove_unused_variables_subprocess() {
    let subprocess = SubprocessFragment::new(
        VarExprFragment::new("a", Type::Generic).to_frag()
    );

    let mut ast = BlockFragment::new(bash_code!({
        a = 10;
        b = 20;
        syntax(subprocess.to_frag());
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert!(matches!(block.statements[1].clone(), FragmentKind::Subprocess(_)));
}

#[test]
fn test_remove_unused_variables_transitive() {
    let mut ast = BlockFragment::new(bash_code!({
        a = 10;
        b = a;
        c = b;
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 0);
}

#[test]
fn test_remove_unused_variables_in_list() {
    let list = ListFragment::new(vec![
        VarExprFragment::new("a", Type::Generic).to_frag(),
        VarExprFragment::new("b", Type::Generic).to_frag(),
    ]).with_spaces();

    let mut ast = BlockFragment::new(bash_code!({
        a = 10;
        b = 20;
        c = 30;
        syntax(list.to_frag());
    }), true).to_frag();

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

    let index_expr = VarExprFragment::from_stmt(&index_stmt);
    let mut array_expr = VarExprFragment::from_stmt(&array_stmt);

    // TODO: Replece this with `with_index_by_value` when #702 gets merged
    array_expr.index = Some(Box::new(VarIndexValue::Index(index_expr.to_frag())));

    let mut ast = BlockFragment::new(bash_code!({
        syntax(array_stmt.to_frag());
        syntax(index_stmt.to_frag());
        unused = 0;
        syntax(array_expr.to_frag());
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "array");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "index");
    assert!(matches!(block.statements[2].clone(), FragmentKind::VarExpr(_)));
}

#[test]
fn test_remove_unused_variables_multiple_references() {
    let mut ast = BlockFragment::new(bash_code!({
        a = 10;
        b = 20;
        a;
        a;
        a;
    }), true).to_frag();

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
    // Build the complete AST
    let mut ast = BlockFragment::new(bash_code!({
        outer = 10;
        outer_unused = 20;
        {
            inner = outer;
            inner_unused = outer_unused;
            inner;
        }
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "outer");
    let inner_block = unwrap_fragment!(block.statements[1].clone(), Block);
    assert_eq!(inner_block.statements.len(), 2);
    assert_eq!(unwrap_fragment!(inner_block.statements[0].clone(), VarStmt).get_name(), "inner");
    assert_eq!(unwrap_fragment!(inner_block.statements[1].clone(), VarExpr).get_name(), "inner");
}

#[test]
fn test_nested_conditional_scope_transitive() {
    // Build the complete AST
    let mut ast = BlockFragment::new(bash_code!({
        a = 10;
        b = a;
        if {
            b = a;
            b = 24;
        }
        b;
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    dbg!(block.clone());
    assert_eq!(block.statements.len(), 4);
    assert_eq!(unwrap_fragment!(block.statements[0].clone(), VarStmt).get_name(), "a");
    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "b");
    let inner_block = unwrap_fragment!(block.statements[2].clone(), Block);
    let inner_var = unwrap_fragment!(inner_block.statements[0].clone(), VarStmt);
    assert_eq!(inner_var.get_name(), "b");
    assert_eq!(unwrap_fragment!(*inner_var.value, Raw).value, "24");
    assert_eq!(unwrap_fragment!(block.statements[3].clone(), VarExpr).get_name(), "b");
}

#[test]
fn test_nested_conditional_scope_transitive_single_assignment() {
    // Build the complete AST
    let mut ast = BlockFragment::new(bash_code!({
        a = 12;
        a = 10;
        b = a;
        c = a;
        if {
            c = b;
        }
        c;
    }), true).to_frag();

    remove_unused_variables(&mut ast);

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 5);

    let first_var = unwrap_fragment!(block.statements[0].clone(), VarStmt);
    assert_eq!(first_var.get_name(), "a");
    assert_eq!(unwrap_fragment!(*first_var.value, Raw).value, "10");

    assert_eq!(unwrap_fragment!(block.statements[1].clone(), VarStmt).get_name(), "b");
    assert_eq!(unwrap_fragment!(block.statements[2].clone(), VarStmt).get_name(), "c");

    let inner_block = unwrap_fragment!(block.statements[3].clone(), Block);
    assert_eq!(inner_block.statements.len(), 1);
    unwrap_fragment!(inner_block.statements[0].clone(), VarStmt);

    assert_eq!(unwrap_fragment!(block.statements[4].clone(), VarExpr).get_name(), "c");
}
