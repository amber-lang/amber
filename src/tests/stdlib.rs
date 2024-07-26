#![cfg(test)]
extern crate test_generator;
use test_generator::test_resources;
use crate::compiler::AmberCompiler;
use crate::test_amber;
use crate::tests::compile_code;
use std::path::Path;
use std::fs;
use std::time::Duration;
use std::process::{Command, Stdio};

/*
 * Autoload the Amber test files for stdlib and match the output with the output.txt file
 */
#[test_resources("src/tests/stdlib/*.ab")]
fn stdlib_test(input: &str) {
    let code = fs::read_to_string(input)
    .expect(&format!("Failed to open {input} test file"));

    let mut is_output = false;
    let mut output = "".to_owned();
    for line in code.lines() {
        if line.starts_with("// Output") {
            is_output = true;
            continue;
        } else if line.is_empty() && _is_output {
            is_output = false;
            break;
        }

        if is_output {
            if ! output.is_empty() {
                output.push_str("\n");
            }
            output.push_str(&line.replace("//", "").trim());
        }
    }

    if output.is_empty() {
        output = match Path::new(&input.replace(".ab", ".output.txt")).exists() {
            true => fs::read_to_string(input.replace(".ab", ".output.txt"))
            .expect(&format!("Failed to open {input}.output.txt file")),
            _ => "Succeded".to_string()
        };
    }

    test_amber!(code, output);
}

fn http_server() {
    use tiny_http::{Server, Response};
    
    let server = Server::http("127.0.0.1:8081").expect("Can't bind to 127.0.0.1:8081");
    for req in server.incoming_requests() {
        req.respond(Response::from_string("ok")).expect("Can't respond");
        break;
    }
}

#[test]
fn exit() {
    let code = fs::read_to_string("src/tests/stdlib/no_output/exit.ab")
    .expect(&format!("Failed to open stdlib/no_output/exit.ab test file"));

    let code = compile_code(code);
    let mut cmd = Command::new("bash")
        .arg("-c").arg(code)
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn().expect("Couldn't spawn bash");

    assert_eq!(cmd.wait().expect("Couldn't wait for bash to execute").code(), Some(37));
}

#[test]
fn download() {
    let server = std::thread::spawn(http_server);

    let code = fs::read_to_string("src/tests/stdlib/no_output/download.ab")
    .expect(&format!("Failed to open stdlib/no_output/download.ab test file"));

    test_amber!(code, "ok");

    std::thread::sleep(Duration::from_millis(150));
    assert!(server.is_finished(), "Server has not stopped!");
}
