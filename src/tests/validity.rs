use crate::compiler::AmberCompiler;

macro_rules! test_amber {
    ($code:expr, $result:expr) => {
        {
            // RDC is disabled in these tests because it will be tested later and could break things
            match AmberCompiler::new($code.to_string(), None, false).test_eval() {
                Ok(result) => assert_eq!(result.trim(), $result),
                Err(err) => panic!("ERROR: {}", err.message.unwrap())
            }

        }
    };
}

#[test]
fn hello_world() {
    test_amber!("echo \"Hello World\"", "Hello World");
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
    test_amber!("echo \"Hello World\"", "Hello World");
}

#[test]
fn text_escaped() {
    test_amber!("echo \"Hello \\\"World\\\"\"", "Hello \"World\"");
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
        let x = \"welcome {\"to\"} wonderful {\"world\"}\"
        echo x
    ";
    test_amber!(code, "welcome to wonderful world");
}

#[test]
fn text_escaped_interpolated() {
    let code = "
        let x = \"World\"
        echo \"Hello \\\"{x}\\\"\"
    ";
    test_amber!(code, "Hello \"World\"");
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
fn parenthesis() {
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
        echo $echo {$echo Hello World$ failed {}}$ failed {}
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn command_inception() {
    let code = "
        $echo {$echo {$echo {$echo Hello World$ failed {}}$ failed {}}$ failed {}}$ failed {}
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn comment() {
    let code = "
        // this is a comment
        let a = 42 // this is a comment as well
    ";
    test_amber!(code, "");
}

#[test]
fn compare_eq_texts() {
    let code = "
        let x = \"Hello World\"
        let y = \"Hello World\"
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
        let y = 42
        if {
            x == y {
                echo x
            }
        }
    ";
    test_amber!(code, "42");
}

#[test]
fn if_statement_chain_else() {
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
        let x = \"Hello \"
        x += \"World\"
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
        if x == y: echo x
        else: echo y
    ";
    test_amber!(code, "24");
}

#[test]
fn if_statements_else_singleline() {
    let code = "
        let x = 42
        let y = 24
        if x == y: echo x
        else: echo y
    ";
    test_amber!(code, "24");
}

#[test]
fn if_statement_chain_singleline() {
    let code = "
        let x = 42
        let y = 24
        if {
            x == y: echo x
            else: echo y
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
        main {
            loop {
                a += 1
                if a == 5 {
                    continue
                }
                $printf \"{a} \"$?
                if a == 10 {
                    break
                }
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
            return \"Hello World\"
        }
        echo test()
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn function_with_args() {
    let code = "
        fun test(a, b) {
            return \"{a} {b}\"
        }
        echo test(\"Hello\", \"World\")
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn function_with_args_different_types() {
    let code = "
        fun test(a, b) {
            return a + b
        }
        echo test(\"Hello\", \"World\")
        echo test(11, 42)
    ";
    test_amber!(code, "HelloWorld\n53");
}

#[test]
fn function_with_typed_args() {
    let code = "
        fun test(a: Num, b: Num) {
            return a + b
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
        test(11, \"Hello\")
    ";
    test_amber!(code, "11\nHello");
}

#[test]
fn function_with_typed_args_text() {
    let code = "
        pub fun test(a: Text, b: Text) {
            echo a + b
        }
        test(\"Hello\", \"World\")
    ";
    test_amber!(code, "HelloWorld");
}

#[test]
fn import_existing_file() {
    let code = "
        import * from \"test_files/str/trim.ab\"
        echo trim(\"    Hello World     \")
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn import_existing_nested_file() {
    let code = "
        import * from \"test_files/is_even.ab\"
        echo is_even(10)
    ";
    test_amber!(code, "even");
}

#[test]
fn public_import() {
    let code = "
        import * from \"test_files/is_even.ab\"
        echo trim(\" test \")
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
            a = \"Hello World\"
        }

        let a = \"Goodbye World\"
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

#[test]
fn null() {
    let code = "
        let a = null
        echo a
    ";
    test_amber!(code, "");
}

#[test]
fn failed() {
    let code = "
        import { sum } from \"std\"
        let requirements = [true, true, true]

        main {
            silent {
                $make -v$ failed: requirements[0] = false
                $gcc -v$ failed: requirements[1] = false
                $you don\\'t have this$ failed: requirements[2] = false
            }

            if sum(requirements as [Num]) == 3 {
                echo \"All requirements are met\"
            } else {
                echo \"Requirements not met\"
                fail
            }
        }
    ";
    test_amber!(code, "Requirements not met");
}

#[test]
fn nested_failed() {
    let code = "
        fun inner_c() {
            echo \"inner_c\"
            fail
        }

        fun inner_b() {
            inner_c()?
            echo \"inner_b\"
        }

        fun inner_a() {
            inner_b()?
            echo \"inner_a\"
        }

        main {
            inner_a()?
            echo \"main\"
        }
    ";
    test_amber!(code, "inner_c");
}

#[test]
fn silent() {
    let code = "
        main {
            silent {
                echo \"Hello World\"
                $non-existent command$?
            }
        }
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn status() {
    let code = "
        silent $non-existent command$ failed {
            echo status
            echo status
        }
    ";
    test_amber!(code, "127\n127");
}

#[test]
fn test_std_library() {
    let code = "
        import * from \"std\"

        main {
            // Split a string into a list of strings (characters)
            echo chars(\"hello world\")
            // Split a string into a list of strings by a delimiter
            echo split(\"hello world\", \"l\")
            // Split a multiline string to lines of string
            loop line in lines(\"hello\nworld\") {
                echo line
            }
            // Split a multiline string into a list of string by one or more spaces
            loop word in words(\"hello   world ciao     mondo\") {
                echo word
            }
            // Split a joined string into a list of string
            loop word in words(join([\"hello\", \"world\"], \" \")) {
                echo word
            }
            // Join a list of strings into a string
            echo join(split(\"hello world\", \"l\"), \"l\")
            // Transform string into a lowercase string
            echo lower(\"HELLO WORLD\")
            // Transform string into an uppercase string
            echo upper(\"hello world\")
            // Trim whitespace from the beginning and end of a string
            echo \"|{trim(\" hello world \")}|\"
            // Trim whitespace from the beginning of a string
            echo \"|{trim_left(\" hello world \")}|\"
            // Trim whitespace from the end of a string
            echo \"|{trim_right(\" hello world \")}|\"
            // Get the length of a string or list
            echo len(\"hello world\")
            echo len([1,2,3])
            // Replace all occurrences of a substring with another substring
            echo replace(\"hello world\", \"world\", \"universe\")
            // Parse text into a number
            echo parse(\"123\")?
            // Parse text into a number - do some code if failed
            parse(\"XDDDDabc123\") failed {
                echo \"Parsing Failed\"
            }
            // Check if array includes certain word
            echo includes([\"hello\", \"world\"], \"hello\")
            // Check if array does not include certain word
            echo includes([\"hello\", \"world\"], \"other\")
        }
    ";
    test_amber!(code, vec![
        "h e l l o   w o r l d",
        "he  o wor d",
        "hello",
        "world",
        "hello",
        "world",
        "ciao",
        "mondo",
        "hello",
        "world",
        "hello world",
        "hello world",
        "HELLO WORLD",
        "|hello world|",
        "|hello world |",
        "| hello world|",
        "11",
        "3",
        "hello universe",
        "123",
        "Parsing Failed",
        "1",
        "0",
    ].join("\n"));
}

#[test]
fn chained_modifiers() {
    let code = "
        unsafe silent {
            echo \"Hello World\"
            $non-existent command$
        }
        // Test for expression
        let _ = silent unsafe $non-existent command$
        // Test for single statement
        silent unsafe $non-existent command$
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn chained_modifiers_commands() {
    let code = "
        unsafe silent {
            echo \"Hello World\"
            $non-existent command$
        }
        // Test for expression
        let _ = silent unsafe $non-existent command$
        // Test for single statement
        silent unsafe $non-existent command$
    ";
    test_amber!(code, "Hello World");
}

#[test]
fn chained_modifiers_functions() {
    let code = "
        fun foo(a) {
            echo a
            fail 1
        }

        fun bar() {
            echo \"this should not appear\"
        }

        unsafe foo(\"one\")
        unsafe {
            foo(\"two\")
        }
        unsafe silent foo(\"this should not appear\")
        silent bar()
    ";
    test_amber!(code, "one\ntwo");
}

#[test]
fn variable_ref_set_text() {
    let code = "
        fun foo(ref a) {
            a = \"one\"
        }

        let a = \"two\"
        foo(a)
        echo a
    ";
    test_amber!(code, "one");
}

#[test]
fn variable_ref_set_num() {
    let code = "
        fun foo(ref a) {
            a = 42
        }

        let a = 24
        foo(a)
        echo a
    ";
    test_amber!(code, "42");
}

#[test]
fn variable_ref_set_bool() {
    let code = "
        fun foo(ref a) {
            a = false
        }

        let a = true
        foo(a)
        echo a
    ";
    test_amber!(code, "0");
}

#[test]
fn variable_ref_set_array() {
    let code = "
        fun foo(ref a) {
            a = [1, 2, 3]
        }

        let a = [3, 2, 1]
        foo(a)
        echo a
    ";
    test_amber!(code, "1 2 3");
}

#[test]
fn variable_ref_add_shorthand_text() {
    let code = "
        fun foo(ref a) {
            a += \"one\"
        }

        let a = \"two\"
        foo(a)
        echo a
    ";
    test_amber!(code, "twoone");
}

#[test]
fn variable_ref_add_shorthand_num() {
    let code = "
        fun foo(ref a) {
            a += 12
        }

        let a = 24
        foo(a)
        echo a
    ";
    test_amber!(code, "36");
}

#[test]
fn variable_ref_add_shorthand_array() {
    let code = "
        fun foo(ref a) {
            a += [4, 5, 6]
        }

        let a = [1, 2, 3]
        foo(a)
        echo a
    ";
    test_amber!(code, "1 2 3 4 5 6");
}

#[test]
fn variable_ref_sub_shorthand_num() {
    let code = "
        fun foo(ref a) {
            a -= 12
        }

        let a = 36
        foo(a)
        echo a
    ";
    test_amber!(code, "24");
}

#[test]
fn variable_ref_mul_shorthand_num() {
    let code = "
        fun foo(ref a) {
            a *= 2
        }

        let a = 6
        foo(a)
        echo a
    ";
    test_amber!(code, "12");
}

#[test]
fn variable_ref_div_shorthand_num() {
    let code = "
        fun foo(ref a) {
            a /= 3
        }

        let a = 15
        foo(a)
        echo a
    ";
    test_amber!(code, "5");
}

#[test]
fn variable_ref_mod_shorthand_num() {
    let code = "
        fun foo(ref a) {
            a %= 5
        }

        let a = 17
        foo(a)
        echo a
    ";
    test_amber!(code, "2");
}

#[test]
fn variable_ref_add_arithmetic_text() {
    let code = "
        fun foo(ref a, b) {
            a = a + b
        }

        let a = \"two\"
        foo(a, \"one\")
        echo a
    ";
    test_amber!(code, "twoone");
}

#[test]
fn variable_ref_sub_arithmetic_num() {
    let code = "
        fun foo(ref a, b) {
            a = a - b
        }

        let a = 36
        foo(a, 12)
        echo a
    ";
    test_amber!(code, "24");
}


#[test]
fn variable_ref_mul_arithmetic_num() {
    let code = "
        fun foo(ref a, b) {
            a = a * b
        }

        let a = 6
        foo(a, 2)
        echo a
    ";
    test_amber!(code, "12");
}

#[test]
fn variable_ref_div_arithmetic_num() {
    let code = "
        fun foo(ref a, b) {
            a = a / b
        }

        let a = 15
        foo(a, 3)
        echo a
    ";
    test_amber!(code, "5");
}

#[test]
fn variable_ref_mod_arithmetic_num() {
    let code = "
        fun foo(ref a, b) {
            a = a % b
        }

        let a = 17
        foo(a, 5)
        echo a
    ";
    test_amber!(code, "2");
}

#[test]
fn variable_ref_command() {
    let code = "
        fun foo(ref a) {
            a = $echo Test$?
        }

        let a = \"\"
        unsafe foo(a)
        echo a
    ";
    test_amber!(code, "Test");
}

#[test]
fn variable_ref_function_invocation() {
    let code = "
        fun reverse(input: Text): Text {
            return unsafe $echo {input} | rev$
        }

        fun foo(ref a) {
            a = reverse(\"mars\")
        }

        let a = \"\"
        unsafe foo(a)
        echo a
    ";
    test_amber!(code, "\"sram\"");
}
