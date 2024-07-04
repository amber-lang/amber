#![cfg(test)]
extern crate test_generator;
use test_generator::test_resources;
use crate::compiler::AmberCompiler;
use crate::test_amber;
use crate::tests::compile_code;
use std::fs;
use std::io::Write;
use std::time::Duration;
use std::process::{Command, Stdio};

/*
 * Autoload the Amber test files for stdlib and match the output with the output.txt file
 */
#[test_resources("src/tests/stdlib/*.ab")]
fn amber_test(input: &str) {
    let code = fs::read_to_string(input)
    .expect(&format!("Failed to open {input} test file"));

    let output = fs::read_to_string(input.replace(".ab", ".output.txt"))
    .expect(&format!("Failed to open {input}.output.txt file"));

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
fn input() {
    let prompt_message = "Please enter your name:";

    let code = fs::read_to_string("src/tests/stdlib/no_output/input.ab")
    .expect(&format!("Failed to open stdlib/no_output/input.ab test file"));

    let input = "Amber";
    let expected_output = format!("{}Hello, {}", prompt_message, input);

    let compiled_code = compile_code(code);

    let mut child = Command::new("bash")
        .arg("-c")
        .arg(compiled_code)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    let output_str = String::from_utf8(output.stdout).expect("Failed to convert stdout to String");

    assert_eq!(output_str.trim_end_matches('\n'), expected_output);
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

    let code = fs::read_to_string("src/tests/stdlib_no-output/download.ab")
    .expect(&format!("Failed to open stdlib_no-output/download.ab test file"));

    test_amber!(code, "ok");

    std::thread::sleep(Duration::from_millis(150));
    assert!(server.is_finished(), "Server has not stopped!");
}
