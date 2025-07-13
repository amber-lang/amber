use crate::tests::translation::translate_amber_code;
use crate::modules::prelude::*;
use crate::translate::compute::{translate_bc_sed_computation, ArithOp};

#[test]
fn test_translation_add_int() {
    let code = r"
        echo 15 + 45
        echo 39 + 21 + 80
        echo 3 + 9 + 648 + 1232
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 15 + 45
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                RawFragment::new("15").to_frag(),
                ArithOp::Add,
                RawFragment::new("45").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 39 + 21 + 80
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    RawFragment::new("39").to_frag(),
                    ArithOp::Add,
                    RawFragment::new("21").to_frag()
                ).to_frag(),
                ArithOp::Add,
                RawFragment::new("80").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 3 + 9 + 648 + 1232
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    ArithmeticFragment::new(
                        RawFragment::new("3").to_frag(),
                        ArithOp::Add,
                        RawFragment::new("9").to_frag()
                    ).to_frag(),
                    ArithOp::Add,
                    RawFragment::new("648").to_frag()
                ).to_frag(),
                ArithOp::Add,
                RawFragment::new("1232").to_frag()
            ).to_frag(),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_add_num() {
    let code = r"
        echo 15.15 + 45.15
        echo 39.45 + 21.18 + 80.12
        echo 3.0 + 9.0 + 648.0 + 12.23
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 15.15 + 45.15
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Add,
                RawFragment::new("15.15").to_frag(),
                RawFragment::new("45.15").to_frag()
            ),
        ]).to_frag(),
        
        // echo 39.45 + 21.18 + 80.12
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Add,
                translate_bc_sed_computation(
                    ArithOp::Add,
                    RawFragment::new("39.45").to_frag(),
                    RawFragment::new("21.18").to_frag()
                ),
                RawFragment::new("80.12").to_frag()
            ),
        ]).to_frag(),
        
        // echo 3.0 + 9.0 + 648.0 + 12.23
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Add,
                translate_bc_sed_computation(
                    ArithOp::Add,
                    translate_bc_sed_computation(
                        ArithOp::Add,
                        RawFragment::new("3.0").to_frag(),
                        RawFragment::new("9.0").to_frag()
                    ),
                    RawFragment::new("648.0").to_frag()
                ),
                RawFragment::new("12.23").to_frag()
            ),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_sub_int() {
    let code = r"
        echo 21 - 7
        echo 2 - 1 - 3
        echo 1 - 3 - 2 - 12
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 21 - 7
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                RawFragment::new("21").to_frag(),
                ArithOp::Sub,
                RawFragment::new("7").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 2 - 1 - 3
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    RawFragment::new("2").to_frag(),
                    ArithOp::Sub,
                    RawFragment::new("1").to_frag()
                ).to_frag(),
                ArithOp::Sub,
                RawFragment::new("3").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 1 - 3 - 2 - 12
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    ArithmeticFragment::new(
                        RawFragment::new("1").to_frag(),
                        ArithOp::Sub,
                        RawFragment::new("3").to_frag()
                    ).to_frag(),
                    ArithOp::Sub,
                    RawFragment::new("2").to_frag()
                ).to_frag(),
                ArithOp::Sub,
                RawFragment::new("12").to_frag()
            ).to_frag(),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_sub_num() {
    let code = r"
        echo 21 - 7
        echo 2 - 1 - 3
        echo 1.5 - 3.5 - 2.5
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 21 - 7
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                RawFragment::new("21").to_frag(),
                ArithOp::Sub,
                RawFragment::new("7").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 2 - 1 - 3
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    RawFragment::new("2").to_frag(),
                    ArithOp::Sub,
                    RawFragment::new("1").to_frag()
                ).to_frag(),
                ArithOp::Sub,
                RawFragment::new("3").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 1.5 - 3.5 - 2.5
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Sub,
                translate_bc_sed_computation(
                    ArithOp::Sub,
                    RawFragment::new("1.5").to_frag(),
                    RawFragment::new("3.5").to_frag()
                ),
                RawFragment::new("2.5").to_frag()
            ),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_mul_int() {
    let code = r"
        echo 5 * 3
        echo 2 * 4 * 6
        echo 1 * 2 * 3 * 4
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 5 * 3
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                RawFragment::new("5").to_frag(),
                ArithOp::Mul,
                RawFragment::new("3").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 2 * 4 * 6
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    RawFragment::new("2").to_frag(),
                    ArithOp::Mul,
                    RawFragment::new("4").to_frag()
                ).to_frag(),
                ArithOp::Mul,
                RawFragment::new("6").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 1 * 2 * 3 * 4
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    ArithmeticFragment::new(
                        RawFragment::new("1").to_frag(),
                        ArithOp::Mul,
                        RawFragment::new("2").to_frag()
                    ).to_frag(),
                    ArithOp::Mul,
                    RawFragment::new("3").to_frag()
                ).to_frag(),
                ArithOp::Mul,
                RawFragment::new("4").to_frag()
            ).to_frag(),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_mul_num() {
    let code = r"
        echo 5.5 * 3.2
        echo 2.1 * 4.0 * 6.3
        echo 1.0 * 2.5 * 3.1 * 4.7
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 5.5 * 3.2
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Mul,
                RawFragment::new("5.5").to_frag(),
                RawFragment::new("3.2").to_frag()
            ),
        ]).to_frag(),
        
        // echo 2.1 * 4.0 * 6.3
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Mul,
                translate_bc_sed_computation(
                    ArithOp::Mul,
                    RawFragment::new("2.1").to_frag(),
                    RawFragment::new("4.0").to_frag()
                ),
                RawFragment::new("6.3").to_frag()
            ),
        ]).to_frag(),
        
        // echo 1.0 * 2.5 * 3.1 * 4.7
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Mul,
                translate_bc_sed_computation(
                    ArithOp::Mul,
                    translate_bc_sed_computation(
                        ArithOp::Mul,
                        RawFragment::new("1.0").to_frag(),
                        RawFragment::new("2.5").to_frag()
                    ),
                    RawFragment::new("3.1").to_frag()
                ),
                RawFragment::new("4.7").to_frag()
            ),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_div_int() {
    let code = r"
        echo 10 / 2
        echo 20 / 4 / 2
        echo 100 / 5 / 2 / 2
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 10 / 2
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                RawFragment::new("10").to_frag(),
                ArithOp::Div,
                RawFragment::new("2").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 20 / 4 / 2
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    RawFragment::new("20").to_frag(),
                    ArithOp::Div,
                    RawFragment::new("4").to_frag()
                ).to_frag(),
                ArithOp::Div,
                RawFragment::new("2").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 100 / 5 / 2 / 2
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    ArithmeticFragment::new(
                        RawFragment::new("100").to_frag(),
                        ArithOp::Div,
                        RawFragment::new("5").to_frag()
                    ).to_frag(),
                    ArithOp::Div,
                    RawFragment::new("2").to_frag()
                ).to_frag(),
                ArithOp::Div,
                RawFragment::new("2").to_frag()
            ).to_frag(),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_div_num() {
    let code = r"
        echo 10.5 / 2.1
        echo 20.4 / 4.0 / 2.0
        echo 100.0 / 5.0 / 2.5 / 2.0
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 10.5 / 2.1
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Div,
                RawFragment::new("10.5").to_frag(),
                RawFragment::new("2.1").to_frag()
            ),
        ]).to_frag(),
        
        // echo 20.4 / 4.0 / 2.0
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Div,
                translate_bc_sed_computation(
                    ArithOp::Div,
                    RawFragment::new("20.4").to_frag(),
                    RawFragment::new("4.0").to_frag()
                ),
                RawFragment::new("2.0").to_frag()
            ),
        ]).to_frag(),
        
        // echo 100.0 / 5.0 / 2.5 / 2.0
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Div,
                translate_bc_sed_computation(
                    ArithOp::Div,
                    translate_bc_sed_computation(
                        ArithOp::Div,
                        RawFragment::new("100.0").to_frag(),
                        RawFragment::new("5.0").to_frag()
                    ),
                    RawFragment::new("2.5").to_frag()
                ),
                RawFragment::new("2.0").to_frag()
            ),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_mod_int() {
    let code = r"
        echo 10 % 3
        echo 20 % 6 % 3
        echo 100 % 7 % 3 % 2
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 10 % 3
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                RawFragment::new("10").to_frag(),
                ArithOp::Modulo,
                RawFragment::new("3").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 20 % 6 % 3
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    RawFragment::new("20").to_frag(),
                    ArithOp::Modulo,
                    RawFragment::new("6").to_frag()
                ).to_frag(),
                ArithOp::Modulo,
                RawFragment::new("3").to_frag()
            ).to_frag(),
        ]).to_frag(),
        
        // echo 100 % 7 % 3 % 2
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            ArithmeticFragment::new(
                ArithmeticFragment::new(
                    ArithmeticFragment::new(
                        RawFragment::new("100").to_frag(),
                        ArithOp::Modulo,
                        RawFragment::new("7").to_frag()
                    ).to_frag(),
                    ArithOp::Modulo,
                    RawFragment::new("3").to_frag()
                ).to_frag(),
                ArithOp::Modulo,
                RawFragment::new("2").to_frag()
            ).to_frag(),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}

#[test]
fn test_translation_mod_num() {
    let code = r"
        echo 10.5 % 3.2
        echo 20.4 % 6.0 % 3.1
        echo 100.0 % 7.5 % 3.0 % 2.5
    ";

    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");

    // Create the expected AST structure explicitly
    let expected = BlockFragment::new(vec![
        // echo 10.5 % 3.2
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Modulo,
                RawFragment::new("10.5").to_frag(),
                RawFragment::new("3.2").to_frag()
            ),
        ]).to_frag(),
        
        // echo 20.4 % 6.0 % 3.1
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Modulo,
                translate_bc_sed_computation(
                    ArithOp::Modulo,
                    RawFragment::new("20.4").to_frag(),
                    RawFragment::new("6.0").to_frag()
                ),
                RawFragment::new("3.1").to_frag()
            ),
        ]).to_frag(),
        
        // echo 100.0 % 7.5 % 3.0 % 2.5
        ListFragment::new(vec![
            RawFragment::new("echo ").to_frag(),
            translate_bc_sed_computation(
                ArithOp::Modulo,
                translate_bc_sed_computation(
                    ArithOp::Modulo,
                    translate_bc_sed_computation(
                        ArithOp::Modulo,
                        RawFragment::new("100.0").to_frag(),
                        RawFragment::new("7.5").to_frag()
                    ),
                    RawFragment::new("3.0").to_frag()
                ),
                RawFragment::new("2.5").to_frag()
            ),
        ]).to_frag(),
    ], true).to_frag();

    assert_eq!(ast, expected);
}
