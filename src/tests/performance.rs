use crate::compiler::AmberCompiler;
use std::time::Duration;

const REPS: u32 = 10;

// The following tests calculate the time it takes to compile the code and execute by a bash interpreter
macro_rules! test_time_amber {
    ($code:expr, $result:expr) => {
        {
            let mut times = Vec::with_capacity(REPS as usize);
            for _ in 0..REPS {
                let time = std::time::Instant::now();
                match AmberCompiler::new($code.to_string(), None).test_eval() {
                    Ok(_) => times.push(time.elapsed()),
                    Err(err) => panic!("ERROR: {}", err.message.unwrap())
                }
            }
            let avg = times.iter().fold(Duration::new(0, 0), |acc, x| acc + *x) / REPS;
            if avg > $result {
                panic!("ERROR: Took too long to execute {}ms", avg.as_millis());
            }
        }
    };
}

#[test]
fn hello_world() {
    test_time_amber!("echo 'Hello World'", Duration::from_millis(20));
}

#[test]
fn add() {
    test_time_amber!("echo 15 + 45", Duration::from_millis(20));
}

#[test]
fn sub() {
    test_time_amber!("echo 21 - 7", Duration::from_millis(20));
}

#[test]
fn mul() {
    test_time_amber!("echo 3 * 4", Duration::from_millis(50));
}

#[test]
fn div() {
    test_time_amber!("echo 21 / 3", Duration::from_millis(50));
}

#[test]
fn text() {
    test_time_amber!("echo 'Hello World'", Duration::from_millis(20));
}

#[test]
fn bool() {
    test_time_amber!("echo true", Duration::from_millis(20));
    test_time_amber!("echo false", Duration::from_millis(20));
}

#[test]
fn number() {
    test_time_amber!("echo 15", Duration::from_millis(20));
}

#[test]
fn variable() {
    let code = "
        let x = 42
        echo x
    ";
    test_time_amber!(code, Duration::from_millis(20));
}

#[test]
fn if_statement() {
    let code = "
        let x = 42
        if x == 42 {
            echo 'Hello World'
        }
    ";
    test_time_amber!(code, Duration::from_millis(50));
}

#[test]
fn if_else_statement() {
    let code = "
        let x = 42
        if x == 42 {
            echo 'Hello World'
        } else {
            echo 'Goodbye World'
        }
    ";
    test_time_amber!(code, Duration::from_millis(100));
}

#[test]
fn if_else_if_statement() {
    let code = "
        let x = 42
        if {
            x == 42 => echo 'Hello World'
            x == 43 => echo 'Goodbye World'
            else => 'XD'
        }
    ";
    test_time_amber!(code, Duration::from_millis(100));
}

#[test]
fn function() {
    let code = "
        fun test() {
            echo 'Hello World'
        }
        echo test()
    ";
    test_time_amber!(code, Duration::from_millis(50));
}

#[test]
fn function_with_args() {
    let code = "
        fun test(a, b) {
            echo a
            echo b
        }
        echo test('Hello', 'World')
        echo test(11, 42)
    ";
    test_time_amber!(code, Duration::from_millis(100));
}

#[test]
fn function_with_typed_args() {
    let code = "
        fun test(a: Num, b: Num) {
            echo a + b
        }
        echo test(11, 42)
    ";
    test_time_amber!(code, Duration::from_millis(100));
}

#[test]
fn import_existing_file() {
    let code = "
        import * from 'test_files/str/trim.ab'
        echo trim('    Hello World     ')
    ";
    test_time_amber!(code, Duration::from_millis(100));
}
