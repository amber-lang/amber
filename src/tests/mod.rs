use crate::compiler::AmberCompiler;
use difference::{Changeset, Difference};

pub mod cli;
pub mod stdlib;
pub mod unit;
pub mod validity;

const FILE_HEADER: &str = include_str!("../../src/header.sh");

#[macro_export]
macro_rules! test_amber {
    ($code:expr, $result:expr) => {{
        match AmberCompiler::new($code.to_string(), None).test_eval() {
            Ok(result) => assert_eq!(result.trim_end_matches('\n'), $result),
            Err(err) => panic!("ERROR: {}", err.message.unwrap()),
        }
    }};
}

pub fn compile_code<T: Into<String>>(code: T) -> String {
    AmberCompiler::new(code.into(), None).compile().unwrap().1
}

/// Compile code and compare it with the expected value.
/// If any difference is found panic and show a friendly diff
pub fn comp(source: &str, expected: &str) {
    let mut exp = String::with_capacity(500);
    let exp = {
        // Dedent code blocks based on the first line after `r#"`
        let mut lines = expected.lines();
        lines.next();
        let line = lines.next().unwrap_or("");
        let indent = line.chars().take_while(|&c| c.is_whitespace()).count();
        exp.push_str(&line[indent..]);
        exp.push('\n');
        for line in lines {
            if line.len() >= indent {
                exp.push_str(&line[indent..]);
                exp.push('\n');
            } else {
                panic!("Incorrect indentation on line <<<{line}>>>.\nExpected {indent} characters")
            }
        }
        exp.trim_end()
    };

    let compiled = compile_code(source);

    if !compiled.starts_with(FILE_HEADER) {
        panic!("Unexpected header in generated code {compiled}")
    }
    // Ignore newlines before and after the generated code for ease of use
    let compiled = compiled[FILE_HEADER.len()..]
        .trim_end_matches("\n")
        .trim_start_matches("\n");

    if compiled == exp {
        return;
    };

    let changeset = Changeset::new(&compiled, &exp, " ");

    let mut out = String::new();
    let mut cnt = 0;
    for diff in changeset.diffs {
        match diff {
            Difference::Same(ref x) => out.push_str(x),
            Difference::Add(ref x) => out.push_str(&format!("\x1b[32m『{}』\x1b[0m", x)),
            Difference::Rem(ref x) => out.push_str(&format!("\x1b[31m『{}』\x1b[0m", x)),
        }
        cnt += 1;
    }
    println!("--- {} differences found ---", cnt);
    println!("{}", out);
    println!("--- generated code ---");
    println!("{}", compiled);
    println!("=======================================");
    panic!("Unexpected difference");
}
