use crate::compiler::AmberCompiler;

macro_rules! test_amber {
    ($code:expr, $result:expr) => {
        {
            match AmberCompiler::new($code.to_string(), None).test_eval() {
                Ok(result) => assert_eq!(result.trim(), $result),
                Err(err) => panic!("ERROR: {}", err.message.unwrap())
            }

        }
    };
}

#[test]
fn hello_world() {
    test_amber!("echo 'Hello World'", "Hello World");
}

#[test]
fn add() {
    test_amber!("echo 15 + 45", "60");
}

#[test]
fn mul() {
    test_amber!("echo 3 * 4", "12");
}

#[test]
fn div() {
    test_amber!("echo 21 / 3", "7");
}

#[test]
fn sub() {
    test_amber!("echo 21 - 7", "14");
}

#[test]
fn text() {
    test_amber!("echo 'Hello World'", "Hello World");
}

#[test]
fn bool() {
    test_amber!("echo true", "1");
    test_amber!("echo false", "0");
}

#[test]
fn number() {
    test_amber!("echo 15", "15");
}

#[test]
fn variable() {
    let code = "
        let x = 42
        echo x
        x = 21
        echo x
    ";
    test_amber!(code, "42\n21");
}

#[test]
fn nested_string_interp() {
    let code = "
        let x = 'welcome {'to'} wonderful {'world'}'
        echo x
    ";
    test_amber!(code, "welcome to wonderful world");
}

#[test]
fn complex_arithmetic() {
    let code = "
        let x = 21
        let y = 2
        let z = 3
        echo x + y * z
    ";
    test_amber!(code, "27");
}

#[test]
fn very_complex_arithmetic() {
    let code = "
        let x = 21
        let y = 2
        let z = 6
        let a = 4
        echo x + y * z / a
    ";
    test_amber!(code, "24");
}

#[test]
fn paranthesis() {
    let code = "
        let x = 21
        let y = 2
        let z = 6
        let a = 2
        echo (x + y) * z / a
    ";
    test_amber!(code, "69");
}

#[test]
fn command_interpolation() {
    let code = "
        echo $echo {$echo Hello World$}$
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn command_inception() {
    let code = "
        ${${${$echo Hello World$}$}$}$
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn comment() {
    let code = "
        # this is a comment
        let a = 42 # this is a comment as well
    ";
    test_amber!(code, "");
}

#[test]
fn compare_eq_texts() {
    let code = "
        let x = 'Hello World'
        let y = 'Hello World'
        echo x == y
    ";
    test_amber!(code, "1");
}

#[test]
fn compare_eq_numbers() {
    let code = "
        let x = 42
        let y = 42
        echo x == y
    ";
    test_amber!(code, "1");
}

#[test]
fn compare_neq_numbers() {
    let code = "
        let x = 42
        let y = 24
        echo x != y
    ";
    test_amber!(code, "1");
}

#[test]
fn if_statements() {
    let code = "
        let x = 42
        let y = 24
        if x == y {
            echo x
        } else {
            echo y
        }
    ";
    test_amber!(code, "24");
}

#[test]
fn if_statements_else() {
    let code = "
        let x = 42
        let y = 24
        if x == y {
            echo x
        }
        else {
            echo y
        }
    ";
    test_amber!(code, "24");
}

#[test]
fn if_statement_chain() {
    let code = "
        let x = 42
        let y = 24
        if {
            x == y {
                echo x
            }
            else {
                echo y
            }
        }
    ";
    test_amber!(code, "24");
}

#[test]
fn shorthand_add_text() {
    let code = "
        let x = 'Hello '
        x += 'World'
        echo x
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn shorthand_add() {
    let code = "
        let x = 16
        x += 48
        echo x
    ";
    test_amber!(code, "64");
}

#[test]
fn shorthand_sub() {
    let code = "
        let x = 64
        x -= 16
        echo x
    ";
    test_amber!(code, "48");
}

#[test]
fn shorthand_mul() {
    let code = "
        let x = 16
        x *= 4
        echo x
    ";
    test_amber!(code, "64");
}

#[test]
fn shorthand_div() {
    let code = "
        let x = 21
        x /= 3
        echo x
    ";
    test_amber!(code, "7");
}

#[test]
fn if_statements_singleline() {
    let code = "
        let x = 42
        let y = 24
        if x == y => echo x
        else => echo y
    ";
    test_amber!(code, "24");
}

#[test]
fn if_statements_else_singleline() {
    let code = "
        let x = 42
        let y = 24
        if x == y => echo x
        else => echo y
    ";
    test_amber!(code, "24");
}

#[test]
fn if_statement_chain_singleline() {
    let code = "
        let x = 42
        let y = 24
        if {
            x == y => echo x
            else => echo y
        }
    ";
    test_amber!(code, "24");
}

#[test]
fn ternary_conditional_simple() {
    let code = "
        let a = 12 > 24
            then 42
            else 24
        echo a
    ";
    test_amber!(code, "24");
}

#[test]
fn ternary_conditional_inline() {
    let code = "
        let a = 12 > 24 then 42 else 24
        echo a
    ";
    test_amber!(code, "24");
}

#[test]
fn ternary_conditional_nested() {
    let code = "
        let a = 24 > 12
            then (12 > 24
                then 42
                else 24)
            else (12 > 6
                then 24
                else 12)
        echo a
    ";
    test_amber!(code, "24");
}

#[test]
fn infinite_loop() {
    let code = "
        let a = 0
        loop {
            a += 1
            if a == 5 {
                continue
            }
            $printf \"{a} \"$
            if a == 10 {
                break
            }
        }
    ";
    test_amber!(code, "1 2 3 4 6 7 8 9 10");
}

#[test]
fn modulo_operator() {
    let code = "
        let a = 10 % 3
        echo a
    ";
    test_amber!(code, "1");
}

#[test]
fn modulo_shorthand() {
    let code = "
        let a = 10
        a %= 3
        echo a
    ";
    test_amber!(code, "1");
}

#[test]
fn function() {
    let code = "
        fun test() {
            ret 'Hello World'
        }
        echo test()
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn function_with_args() {
    let code = "
        fun test(a, b) {
            ret '{a} {b}'
        }
        echo test('Hello', 'World')
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn function_with_args_different_types() {
    let code = "
        fun test(a, b) {
            ret a + b
        }
        echo test('Hello', 'World')
        echo test(11, 42)
    ";
    test_amber!(code, "HelloWorld\n53");
}

#[test]
fn function_with_typed_args() {
    let code = "
        fun test(a: Num, b: Num) {
            ret a + b
        }
        echo test(11, 42)
    ";
    test_amber!(code, "53");
}

#[test]
fn function_with_typed_different_args() {
    let code = "
        fun test(a: Num, b: Text) {
            echo a
            echo b
        }
        test(11, 'Hello')
    ";
    test_amber!(code, "11\nHello");
}

#[test]
fn function_with_typed_args_text() {
    let code = "
        pub fun test(a: Text, b: Text) {
            echo a + b
        }
        test('Hello', 'World')
    ";
    test_amber!(code, "HelloWorld");
}

#[test]
fn import_existing_file() {
    let code = "
        import * from 'test_files/str/trim.ab'
        echo trim('    Hello World     ')
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn import_existing_nested_file() {
    let code = "
        import * from 'test_files/is_even.ab'
        echo is_even(10)
    ";
    test_amber!(code, "even");
}

#[test]
fn public_import() {
    let code = "
        import * from 'test_files/is_even.ab'
        echo trim(' test ')
    ";
    test_amber!(code, "test");
}

#[test]
fn function_ref_swap() {
    let code = "
        fun swap(ref a, ref b) {
            let temp = a
            a = b
            b = temp
        }
        
        let a = 12
        let b = 24
        
        swap(a, b)
        
        echo a
        echo b
    ";
    test_amber!(code, "24\n12");
}

#[test]
fn function_ref_text_escaped() {
    let code = "
        fun test(ref a) {
            a = 'Hello World'
        }
        
        let a = 'Goodbye World'
        test(a)
        echo a
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn array_init() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        echo a
    ";
    test_amber!(code, "1 2 3 4 5");
}

#[test]
fn array_assign() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        a[2] = 42
        echo a
    ";
    test_amber!(code, "1 2 42 4 5");
}

#[test]
fn array_assign_out_of_bounds() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        a[10] = 42
        echo a
    ";
    test_amber!(code, "1 2 3 4 5 42");
}

#[test]
fn array_pass_by_copy() {
    let code = "
        fun test(a) {
            a[2] = 42
            echo a[2]
        }
        
        let a = [1, 2, 3, 4, 5]
        test(a)
        echo a
    ";
    test_amber!(code, "42\n1 2 3 4 5");
}

#[test]
fn array_pass_by_ref() {
    let code = "
        fun test(ref a) {
            a[2] = 42
            echo a[1]
        }
        
        let a = [1, 2, 3, 4, 5]
        test(a)
        echo a
    ";
    test_amber!(code, "2\n1 2 42 4 5");
}

#[test]
fn add_arrays() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        let b = [6, 7, 8, 9, 10]
        let c = a + b
        echo c
    ";
    test_amber!(code, "1 2 3 4 5 6 7 8 9 10");
}

#[test]
fn shorthand_add_arrays() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        let b = [6, 7, 8, 9, 10]
        a += b
        echo a
    ";
    test_amber!(code, "1 2 3 4 5 6 7 8 9 10");
}

#[test]
fn add_arrays_literal() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        let c = a + [6, 7, 8, 9, 10]
        echo c
    ";
    test_amber!(code, "1 2 3 4 5 6 7 8 9 10");
}

#[test]
fn loop_in() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        loop i in a {
            echo i
        }
    ";
    test_amber!(code, "1\n2\n3\n4\n5");
}

#[test]
fn loop_in_index_value() {
    let code = "
        let a = [1, 2, 3, 4, 5]
        loop i, v in a {
            echo i
            echo v
        }
    ";
    test_amber!(code, "0\n1\n1\n2\n2\n3\n3\n4\n4\n5");
}

#[test]
fn range_loop() {
    let code = "
        loop i in 0..5 {
            echo i
        }
    ";
    test_amber!(code, "0\n1\n2\n3\n4");
}

#[test]
fn range_loop_inclusive() {
    let code = "
        loop i in 0..=5 {
            echo i
        }
    ";
    test_amber!(code, "0\n1\n2\n3\n4\n5");
}
