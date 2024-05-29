use std::{io::{BufWriter, Write}, process::{Command, Stdio}};


/// This mechanism is built to support multiple formatters.
/// 
/// The idea is that amber should find the one installed, verify that its compatible and use the best one possible.
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum BashFormatter {
    /// https://github.com/mvdan/sh
    shfmt
}

impl BashFormatter {
    /// Get all available formatters, ordered by: best ones at the start, worser at the end
    pub fn get_all() -> Vec<BashFormatter> {
        vec![
            BashFormatter::shfmt
        ]
    }

    /// Get available formatter
    pub fn get_available() -> Option<BashFormatter> {
        let all = Self::get_all();
        all.iter().find(|x| x.is_available()).map(|x| *x)
    }

    /// Check if current formatter is present in $PATH
    pub fn is_available(self: &Self) -> bool {
        match self {
            BashFormatter::shfmt =>
                Command::new("shfmt")
                    .arg("--version")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map(|mut x| x.wait())
                    .is_ok()
        }
    }

    /// Format code using the formatter
    pub fn format(self: &Self, code: String) -> String {
        match self {
            BashFormatter::shfmt => {
                let mut command = Command::new("shfmt")
                    .stdout(Stdio::piped())
                    .stdin(Stdio::piped())
                    .arg("-i").arg("4") // identation
                    .arg("-ln").arg("bash") // language
                    .spawn().expect("Couldn't spawn shfmt");

                {
                    let cmd_stdin = command.stdin.as_mut().expect("Couldn't get shfmt's stdin");
                    let mut writer = BufWriter::new(cmd_stdin);
                    writer.write_all(code.as_bytes()).expect("Couldn't write code to shfmt");
                    writer.flush().expect("Couldn't flush shfmt's stdin");
                }

                let res = command.wait_with_output().expect("Couldn't wait for shfmt");

                // let mut stdout = command.stdout.expect("Couldn't get shfmt's stdout");
                // let mut out = String::new();
                // stdout.read_to_string(&mut out).expect("Couldn't read shfmt's output");

                String::from_utf8(res.stdout).expect("shfmt returned non utf-8 output")
            }
        }
    }
}