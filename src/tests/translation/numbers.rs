use crate::{tests::translation::translate_amber_code, unwrap_fragment};
use crate::modules::prelude::*;

#[test]
fn test_translation_add_int() {
    let code = r"
        echo 15 + 45
        echo 39 + 21 + 80
        echo 3 + 9 + 648 + 1232
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Add);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Raw);
        assert_eq!(left.value, "15");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "45");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Add);
        let left_val = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_val.op, ArithOp::Add);
        let left_nested = unwrap_fragment!(left_val.left.unwrap().clone(), Raw);
        assert_eq!(left_nested.value, "39");
        let right_nested = unwrap_fragment!(left_val.right.unwrap().clone(), Raw);
        assert_eq!(right_nested.value, "21");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "80");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Add);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left.op, ArithOp::Add);
        let left_nested_1 = unwrap_fragment!(left.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_nested_1.op, ArithOp::Add);
        let left_nested_2 = unwrap_fragment!(left_nested_1.left.unwrap().clone(), Raw);
        assert_eq!(left_nested_2.value, "3");
        let right_nested_2 = unwrap_fragment!(left_nested_1.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_2.value, "9");
        let right_nested_1 = unwrap_fragment!(left.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_1.value, "648");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "1232");
    }
}

#[test]
fn test_translation_add_num() {
    let code = r"
        echo 15.15 + 45.15
        echo 39.45 + 21.18 + 80.12
        echo 3.0 + 9.0 + 648.0 + 12.23
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(program.values[1].clone(), Raw).value, "15.15");
        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '+' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "45.15");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        // Nested subprocess for "39.45 + 21.18"
        let left_subprocess = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program = unwrap_fragment!(*left_subprocess.fragment.clone(), List);
        assert_eq!(left_program.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program.values[1].clone(), Raw).value, "39.45");
        assert_eq!(unwrap_fragment!(left_program.values[2].clone(), Raw).value, " '+' ");
        assert_eq!(unwrap_fragment!(left_program.values[3].clone(), Raw).value, "21.18");
        assert_eq!(unwrap_fragment!(left_program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '+' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "80.12");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        // Nested subprocess for "3.0 + 9.0 + 648.0"
        let left_subprocess_outer = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program_outer = unwrap_fragment!(*left_subprocess_outer.fragment.clone(), List);
        assert_eq!(left_program_outer.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_outer.values[0].clone(), Raw).value, "echo ");

        // Nested subprocess for "3.0 + 9.0"
        let left_subprocess_inner = unwrap_fragment!(left_program_outer.values[1].clone(), Subprocess);
        let left_program_inner = unwrap_fragment!(*left_subprocess_inner.fragment.clone(), List);
        assert_eq!(left_program_inner.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_inner.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[1].clone(), Raw).value, "3.0");
        assert_eq!(unwrap_fragment!(left_program_inner.values[2].clone(), Raw).value, " '+' ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[3].clone(), Raw).value, "9.0");
        assert_eq!(unwrap_fragment!(left_program_inner.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program_inner.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_inner.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_inner.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(left_program_outer.values[2].clone(), Raw).value, " '+' ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[3].clone(), Raw).value, "648.0");
        assert_eq!(unwrap_fragment!(left_program_outer.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program_outer.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_outer.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_outer.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '+' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "12.23");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
}

#[test]
fn test_translation_sub_int() {
    let code = r"
        echo 21 - 7
        echo 2 - 1 - 3
        echo 1 - 3 - 2 - 12
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Sub);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Raw);
        assert_eq!(left.value, "21");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "7");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Sub);
        let left_val = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_val.op, ArithOp::Sub);
        let left_nested = unwrap_fragment!(left_val.left.unwrap().clone(), Raw);
        assert_eq!(left_nested.value, "2");
        let right_nested = unwrap_fragment!(left_val.right.unwrap().clone(), Raw);
        assert_eq!(right_nested.value, "1");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "3");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Sub);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left.op, ArithOp::Sub);
        let left_nested_1 = unwrap_fragment!(left.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_nested_1.op, ArithOp::Sub);
        let left_nested_2 = unwrap_fragment!(left_nested_1.left.unwrap().clone(), Raw);
        assert_eq!(left_nested_2.value, "1");
        let right_nested_2 = unwrap_fragment!(left_nested_1.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_2.value, "3");
        let right_nested_1 = unwrap_fragment!(left.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_1.value, "2");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "12");
    }
}

#[test]
fn test_translation_sub_num() {
    let code = r"
        echo 21 - 7
        echo 2 - 1 - 3
        echo 1.5 - 3.5 - 2.5
    ";

    let ast = translate_amber_code(code).expect("Couldn\'t translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Sub);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Raw);
        assert_eq!(left.value, "21");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "7");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Sub);
        let left_val = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_val.op, ArithOp::Sub);
        let left_nested = unwrap_fragment!(left_val.left.unwrap().clone(), Raw);
        assert_eq!(left_nested.value, "2");
        let right_nested = unwrap_fragment!(left_val.right.unwrap().clone(), Raw);
        assert_eq!(right_nested.value, "1");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "3");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        // Nested subprocess for "1.5 - 3.5"
        let left_subprocess = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program = unwrap_fragment!(*left_subprocess.fragment.clone(), List);
        assert_eq!(left_program.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program.values[1].clone(), Raw).value, "1.5");
        assert_eq!(unwrap_fragment!(left_program.values[2].clone(), Raw).value, " '-' ");
        assert_eq!(unwrap_fragment!(left_program.values[3].clone(), Raw).value, "3.5");
        assert_eq!(unwrap_fragment!(left_program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '-' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "2.5");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
}

#[test]
fn test_translation_mul_int() {
    let code = r"
        echo 5 * 3
        echo 2 * 4 * 6
        echo 1 * 2 * 3 * 4
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Mul);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Raw);
        assert_eq!(left.value, "5");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "3");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Mul);
        let left_val = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_val.op, ArithOp::Mul);
        let left_nested = unwrap_fragment!(left_val.left.unwrap().clone(), Raw);
        assert_eq!(left_nested.value, "2");
        let right_nested = unwrap_fragment!(left_val.right.unwrap().clone(), Raw);
        assert_eq!(right_nested.value, "4");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "6");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Mul);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left.op, ArithOp::Mul);
        let left_nested_1 = unwrap_fragment!(left.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_nested_1.op, ArithOp::Mul);
        let left_nested_2 = unwrap_fragment!(left_nested_1.left.unwrap().clone(), Raw);
        assert_eq!(left_nested_2.value, "1");
        let right_nested_2 = unwrap_fragment!(left_nested_1.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_2.value, "2");
        let right_nested_1 = unwrap_fragment!(left.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_1.value, "3");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "4");
    }
}

#[test]
fn test_translation_mul_num() {
    let code = r"
        echo 5.5 * 3.2
        echo 2.1 * 4.0 * 6.3
        echo 1.0 * 2.5 * 3.1 * 4.7
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(program.values[1].clone(), Raw).value, "5.5");
        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '*' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "3.2");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        let left_subprocess = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program = unwrap_fragment!(*left_subprocess.fragment.clone(), List);
        assert_eq!(left_program.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program.values[1].clone(), Raw).value, "2.1");
        assert_eq!(unwrap_fragment!(left_program.values[2].clone(), Raw).value, " '*' ");
        assert_eq!(unwrap_fragment!(left_program.values[3].clone(), Raw).value, "4.0");
        assert_eq!(unwrap_fragment!(left_program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '*' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "6.3");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        let left_subprocess_outer = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program_outer = unwrap_fragment!(*left_subprocess_outer.fragment.clone(), List);
        assert_eq!(left_program_outer.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_outer.values[0].clone(), Raw).value, "echo ");

        let left_subprocess_inner = unwrap_fragment!(left_program_outer.values[1].clone(), Subprocess);
        let left_program_inner = unwrap_fragment!(*left_subprocess_inner.fragment.clone(), List);
        assert_eq!(left_program_inner.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_inner.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[1].clone(), Raw).value, "1.0");
        assert_eq!(unwrap_fragment!(left_program_inner.values[2].clone(), Raw).value, " '*' ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[3].clone(), Raw).value, "2.5");
        assert_eq!(unwrap_fragment!(left_program_inner.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program_inner.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_inner.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_inner.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(left_program_outer.values[2].clone(), Raw).value, " '*' ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[3].clone(), Raw).value, "3.1");
        assert_eq!(unwrap_fragment!(left_program_outer.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program_outer.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_outer.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_outer.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '*' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "4.7");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
}

#[test]
fn test_translation_div_int() {
    let code = r"
        echo 10 / 2
        echo 20 / 4 / 2
        echo 100 / 5 / 2 / 2
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Div);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Raw);
        assert_eq!(left.value, "10");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "2");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Div);
        let left_val = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_val.op, ArithOp::Div);
        let left_nested = unwrap_fragment!(left_val.left.unwrap().clone(), Raw);
        assert_eq!(left_nested.value, "20");
        let right_nested = unwrap_fragment!(left_val.right.unwrap().clone(), Raw);
        assert_eq!(right_nested.value, "4");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "2");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Div);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left.op, ArithOp::Div);
        let left_nested_1 = unwrap_fragment!(left.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_nested_1.op, ArithOp::Div);
        let left_nested_2 = unwrap_fragment!(left_nested_1.left.unwrap().clone(), Raw);
        assert_eq!(left_nested_2.value, "100");
        let right_nested_2 = unwrap_fragment!(left_nested_1.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_2.value, "5");
        let right_nested_1 = unwrap_fragment!(left.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_1.value, "2");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "2");
    }
}

#[test]
fn test_translation_div_num() {
    let code = r"
        echo 10.5 / 2.1
        echo 20.4 / 4.0 / 2.0
        echo 100.0 / 5.0 / 2.5 / 2.0
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(program.values[1].clone(), Raw).value, "10.5");
        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '/' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "2.1");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        let left_subprocess = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program = unwrap_fragment!(*left_subprocess.fragment.clone(), List);
        assert_eq!(left_program.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program.values[1].clone(), Raw).value, "20.4");
        assert_eq!(unwrap_fragment!(left_program.values[2].clone(), Raw).value, " '/' ");
        assert_eq!(unwrap_fragment!(left_program.values[3].clone(), Raw).value, "4.0");
        assert_eq!(unwrap_fragment!(left_program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '/' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "2.0");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        let left_subprocess_outer = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program_outer = unwrap_fragment!(*left_subprocess_outer.fragment.clone(), List);
        assert_eq!(left_program_outer.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_outer.values[0].clone(), Raw).value, "echo ");

        let left_subprocess_inner = unwrap_fragment!(left_program_outer.values[1].clone(), Subprocess);
        let left_program_inner = unwrap_fragment!(*left_subprocess_inner.fragment.clone(), List);
        assert_eq!(left_program_inner.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_inner.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[1].clone(), Raw).value, "100.0");
        assert_eq!(unwrap_fragment!(left_program_inner.values[2].clone(), Raw).value, " '/' ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[3].clone(), Raw).value, "5.0");
        assert_eq!(unwrap_fragment!(left_program_inner.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program_inner.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_inner.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_inner.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(left_program_outer.values[2].clone(), Raw).value, " '/' ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[3].clone(), Raw).value, "2.5");
        assert_eq!(unwrap_fragment!(left_program_outer.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(left_program_outer.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_outer.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_outer.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '/' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "2.0");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "-l");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
}

#[test]
fn test_translation_mod_int() {
    let code = r"
        echo 10 % 3
        echo 20 % 6 % 3
        echo 100 % 7 % 3 % 2
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Modulo);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Raw);
        assert_eq!(left.value, "10");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "3");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Modulo);
        let left_val = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_val.op, ArithOp::Modulo);
        let left_nested = unwrap_fragment!(left_val.left.unwrap().clone(), Raw);
        assert_eq!(left_nested.value, "20");
        let right_nested = unwrap_fragment!(left_val.right.unwrap().clone(), Raw);
        assert_eq!(right_nested.value, "6");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "3");
    }
    {
        let arith = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Arithmetic);
        assert_eq!(arith.op, ArithOp::Modulo);
        let left = unwrap_fragment!(arith.left.unwrap().clone(), Arithmetic);
        assert_eq!(left.op, ArithOp::Modulo);
        let left_nested_1 = unwrap_fragment!(left.left.unwrap().clone(), Arithmetic);
        assert_eq!(left_nested_1.op, ArithOp::Modulo);
        let left_nested_2 = unwrap_fragment!(left_nested_1.left.unwrap().clone(), Raw);
        assert_eq!(left_nested_2.value, "100");
        let right_nested_2 = unwrap_fragment!(left_nested_1.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_2.value, "7");
        let right_nested_1 = unwrap_fragment!(left.right.unwrap().clone(), Raw);
        assert_eq!(right_nested_1.value, "3");
        let right = unwrap_fragment!(arith.right.unwrap().clone(), Raw);
        assert_eq!(right.value, "2");
    }
}

#[test]
fn test_translation_mod_num() {
    let code = r"
        echo 10.5 % 3.2
        echo 20.4 % 6.0 % 3.1
        echo 100.0 % 7.5 % 3.0 % 2.5
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    let block = unwrap_fragment!(ast, Block);
    assert_eq!(block.statements.len(), 3);
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[0].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(program.values[1].clone(), Raw).value, "10.5");
        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '%' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "3.2");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, ""); // math_lib_flag is empty for modulo
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[1].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        let left_subprocess = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program = unwrap_fragment!(*left_subprocess.fragment.clone(), List);
        assert_eq!(left_program.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program.values[1].clone(), Raw).value, "20.4");
        assert_eq!(unwrap_fragment!(left_program.values[2].clone(), Raw).value, " '%' ");
        assert_eq!(unwrap_fragment!(left_program.values[3].clone(), Raw).value, "6.0");
        assert_eq!(unwrap_fragment!(left_program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program.values[5].clone(), Raw).value, "");
        assert_eq!(unwrap_fragment!(left_program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '%' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "3.1");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
    {
        let subprocess = unwrap_fragment!(unwrap_fragment!(block.statements[2].clone(), List).values[1].clone(), Subprocess);
        let program = unwrap_fragment!(*subprocess.fragment.clone(), List);
        assert_eq!(program.values.len(), 9);

        assert_eq!(unwrap_fragment!(program.values[0].clone(), Raw).value, "echo ");
        let left_subprocess_outer = unwrap_fragment!(program.values[1].clone(), Subprocess);
        let left_program_outer = unwrap_fragment!(*left_subprocess_outer.fragment.clone(), List);
        assert_eq!(left_program_outer.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_outer.values[0].clone(), Raw).value, "echo ");

        let left_subprocess_inner = unwrap_fragment!(left_program_outer.values[1].clone(), Subprocess);
        let left_program_inner = unwrap_fragment!(*left_subprocess_inner.fragment.clone(), List);
        assert_eq!(left_program_inner.values.len(), 9);
        assert_eq!(unwrap_fragment!(left_program_inner.values[0].clone(), Raw).value, "echo ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[1].clone(), Raw).value, "100.0");
        assert_eq!(unwrap_fragment!(left_program_inner.values[2].clone(), Raw).value, " '%' ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[3].clone(), Raw).value, "7.5");
        assert_eq!(unwrap_fragment!(left_program_inner.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_inner.values[5].clone(), Raw).value, "");
        assert_eq!(unwrap_fragment!(left_program_inner.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_inner.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_inner.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(left_program_outer.values[2].clone(), Raw).value, " '%' ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[3].clone(), Raw).value, "3.0");
        assert_eq!(unwrap_fragment!(left_program_outer.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(left_program_outer.values[5].clone(), Raw).value, "");
        assert_eq!(unwrap_fragment!(left_program_outer.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(left_program_outer.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(left_program_outer.values[8].clone(), Raw).value, "'");

        assert_eq!(unwrap_fragment!(program.values[2].clone(), Raw).value, " '%' ");
        assert_eq!(unwrap_fragment!(program.values[3].clone(), Raw).value, "2.5");
        assert_eq!(unwrap_fragment!(program.values[4].clone(), Raw).value, " | bc ");
        assert_eq!(unwrap_fragment!(program.values[5].clone(), Raw).value, "");
        assert_eq!(unwrap_fragment!(program.values[6].clone(), Raw).value, " | sed '");
        assert_eq!(unwrap_fragment!(program.values[7].clone(), Raw).value, "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//");
        assert_eq!(unwrap_fragment!(program.values[8].clone(), Raw).value, "'");
    }
}
