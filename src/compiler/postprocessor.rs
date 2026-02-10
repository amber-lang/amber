use std::cell::RefCell;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

use itertools::Itertools;
use wildmatch::WildMatchPattern;

#[derive(Debug, Clone)]
pub struct PostProcessor {
    pub name: String,
    pub bin: PathBuf,
    command: Rc<RefCell<Command>>,
}

impl PostProcessor {
    pub fn new<N: Into<String>, B: Into<PathBuf>>(name: N, bin: B) -> Self {
        let name: String = name.into();
        let bin: PathBuf = bin.into();
        let mut command = Command::new(bin.clone());
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let command = Rc::new(RefCell::new(command));
        Self { name, bin, command }
    }

    pub fn cmd(&self) -> Rc<RefCell<Command>> {
        self.command.clone()
    }

    pub fn is_available(&self) -> bool {
        match Command::new(self.bin.clone()).spawn() {
            Ok(mut v) => v.kill().is_ok(),
            Err(_) => false,
        }
    }

    pub fn execute(&self, code: String) -> Result<String, Box<dyn std::error::Error>> {
        if !self.is_available() {
            return Ok(code);
        }

        let mut spawned = self.cmd().borrow_mut().spawn()?;

        // send to stdin
        if let Some(stdin) = spawned.stdin.as_mut() {
            let mut writer = BufWriter::new(stdin);
            writer.write_all(code.as_bytes())?;
            writer.flush()?;
        } else {
            return Err(String::new().into());
        }

        // read from stdout or stderr
        let res = spawned.wait_with_output()?;
        if res.status.success() {
            let stdout = String::from_utf8(res.stdout)?;
            Ok(stdout)
        } else {
            let stderr = String::from_utf8(res.stderr)?;
            Err(stderr.into())
        }
    }

    pub fn get_default() -> Vec<Self> {
        let mut postprocessors = Vec::new();

        let bshchk = PostProcessor::new("bshchk", "bshchk");
        bshchk.cmd().borrow_mut().arg("--ignore-shebang");
        postprocessors.push(bshchk);

        postprocessors
    }

    pub fn filter_default(filters: Vec<WildMatchPattern<'*', '?'>>) -> Vec<Self> {
        let default = Self::get_default();

        default
            .iter()
            .filter(|x| filters.iter().all(|xx| !xx.matches(&x.name)))
            .cloned()
            .collect_vec()
    }
}
