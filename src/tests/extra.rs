#![cfg(test)]
extern crate test_generator;
use super::test_amber;
use super::TestOutcomeTarget;
use crate::tests::compile_code;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;

fn http_server() {
    use tiny_http::{Response, Server};

    let server = Server::http("127.0.0.1:8081").expect("Can't bind to 127.0.0.1:8081");
    if let Some(req) = server.incoming_requests().next() {
        req.respond(Response::from_string("ok"))
            .expect("Can't respond");
    }
}

#[test]
fn exit_with_code() {
    let code = fs::read_to_string("src/tests/validity/no_output/exit_with_code.ab")
        .expect("Failed to open validity/no_output/exit_with_code.ab test file");

    let code = compile_code(code);
    let mut cmd = Command::new("bash")
        .arg("--norc")
        .arg("-c")
        .arg(code)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Couldn't spawn bash");

    assert_eq!(
        cmd.wait()
            .expect("Couldn't wait for bash to execute")
            .code(),
        Some(37)
    );
}

#[test]
fn exit_with_no_code() {
    let code = fs::read_to_string("src/tests/validity/no_output/exit_with_no_code.ab")
        .expect("Failed to open validity/no_output/exit_with_no_code.ab test file");

    let code = compile_code(code);
    let mut cmd = Command::new("bash")
        .arg("-c")
        .arg(code)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Couldn't spawn bash");

    assert_eq!(
        cmd.wait()
            .expect("Couldn't wait for bash to execute")
            .code(),
        Some(0)
    );
}

#[test]
fn download() {
    let server = std::thread::spawn(http_server);

    let code = fs::read_to_string("src/tests/stdlib/no_output/http_file_download.ab")
        .expect("Failed to open stdlib/no_output/http_file_download.ab test file");

    test_amber(&code, "ok", TestOutcomeTarget::Success);

    std::thread::sleep(Duration::from_millis(150));
    assert!(server.is_finished(), "Server has not stopped!");
}
