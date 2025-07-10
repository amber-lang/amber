use crate::{tests::translation::translate_amber_code, unwrap_fragment};
use crate::modules::prelude::*;

#[test]
fn test_translation_shorthand_add_int() {
    let code = r"
        let x = 10
        x += 5
        let y = 20
        y += 15
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand add operation x += 5
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Add);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "x");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "5");

    // Check the shorthand add operation y += 15
    let var_stmt = unwrap_fragment!(block.statements[3].clone(), VarStmt);
    assert_eq!(var_stmt.name, "y");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Add);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "y");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "15");
}

#[test]
fn test_translation_shorthand_add_num() {
    let code = r"
        let x = 10.5
        x += 5.2
        let y = 20.1
        y += 15.7
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand add operation x += 5.2
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let subprocess = unwrap_fragment!(*var_stmt.value.clone(), Subprocess);
    let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
    assert_eq!(program.values.len(), 9);
    
    assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
    let left_var = unwrap_fragment!(program.values[1].clone(), VarExpr);
    assert_eq!(left_var.name, "x");
    assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '+' ");
    assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "5.2");
    assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
    assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
    assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
    assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
    assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
}

#[test]
fn test_translation_shorthand_sub_int() {
    let code = r"
        let x = 20
        x -= 8
        let y = 50
        y -= 25
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand sub operation x -= 8
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Sub);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "x");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "8");

    // Check the shorthand sub operation y -= 25
    let var_stmt = unwrap_fragment!(block.statements[3].clone(), VarStmt);
    assert_eq!(var_stmt.name, "y");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Sub);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "y");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "25");
}

#[test]
fn test_translation_shorthand_sub_num() {
    let code = r"
        let x = 20.5
        x -= 8.2
        let y = 50.8
        y -= 25.3
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand sub operation x -= 8.2
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let subprocess = unwrap_fragment!(*var_stmt.value.clone(), Subprocess);
    let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
    assert_eq!(program.values.len(), 9);
    
    assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
    let left_var = unwrap_fragment!(program.values[1].clone(), VarExpr);
    assert_eq!(left_var.name, "x");
    assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '-' ");
    assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "8.2");
    assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
    assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
    assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
    assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
    assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
}

#[test]
fn test_translation_shorthand_mul_int() {
    let code = r"
        let x = 5
        x *= 4
        let y = 7
        y *= 6
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand mul operation x *= 4
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Mul);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "x");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "4");

    // Check the shorthand mul operation y *= 6
    let var_stmt = unwrap_fragment!(block.statements[3].clone(), VarStmt);
    assert_eq!(var_stmt.name, "y");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Mul);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "y");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "6");
}

#[test]
fn test_translation_shorthand_mul_num() {
    let code = r"
        let x = 5.5
        x *= 4.2
        let y = 7.1
        y *= 6.8
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand mul operation x *= 4.2
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let subprocess = unwrap_fragment!(*var_stmt.value.clone(), Subprocess);
    let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
    assert_eq!(program.values.len(), 9);
    
    assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
    let left_var = unwrap_fragment!(program.values[1].clone(), VarExpr);
    assert_eq!(left_var.name, "x");
    assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '*' ");
    assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "4.2");
    assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
    assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
    assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
    assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
    assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
}

#[test]
fn test_translation_shorthand_div_int() {
    let code = r"
        let x = 20
        x /= 4
        let y = 35
        y /= 5
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand div operation x /= 4
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Div);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "x");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "4");

    // Check the shorthand div operation y /= 5
    let var_stmt = unwrap_fragment!(block.statements[3].clone(), VarStmt);
    assert_eq!(var_stmt.name, "y");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Div);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "y");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "5");
}

#[test]
fn test_translation_shorthand_div_num() {
    let code = r"
        let x = 20.8
        x /= 4.2
        let y = 35.5
        y /= 5.0
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand div operation x /= 4.2
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let subprocess = unwrap_fragment!(*var_stmt.value.clone(), Subprocess);
    let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
    assert_eq!(program.values.len(), 9);
    
    assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
    let left_var = unwrap_fragment!(program.values[1].clone(), VarExpr);
    assert_eq!(left_var.name, "x");
    assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '/' ");
    assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "4.2");
    assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
    assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
    assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
    assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
    assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
}

#[test]
fn test_translation_shorthand_mod_int() {
    let code = r"
        let x = 20
        x %= 7
        let y = 35
        y %= 6
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand mod operation x %= 7
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Modulo);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "x");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "7");

    // Check the shorthand mod operation y %= 6
    let var_stmt = unwrap_fragment!(block.statements[3].clone(), VarStmt);
    assert_eq!(var_stmt.name, "y");
    let arith = unwrap_fragment!(*var_stmt.value.clone(), Arithmetic);
    assert_eq!(arith.op, ArithOp::Modulo);
    let left = unwrap_fragment!(arith.left.unwrap().clone(), VarExpr);
    assert_eq!(left.name, "y");
    let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
    assert_eq!(right.value, "6");
}

#[test]
fn test_translation_shorthand_mod_num() {
    let code = r"
        let x = 20.5
        x %= 7.2
        let y = 35.8
        y %= 6.3
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 4);
    
    // Check the shorthand mod operation x %= 7.2
    let var_stmt = unwrap_fragment!(block.statements[1].clone(), VarStmt);
    assert_eq!(var_stmt.name, "x");
    let subprocess = unwrap_fragment!(*var_stmt.value.clone(), Subprocess);
    let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
    assert_eq!(program.values.len(), 9);
    
    assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
    let left_var = unwrap_fragment!(program.values[1].clone(), VarExpr);
    assert_eq!(left_var.name, "x");
    assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '%' ");
    assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "7.2");
    assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
    assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, ""); // modulo uses empty math_lib_flag
    assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
    assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
    assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
}