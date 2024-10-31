use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};

use itertools::Itertools;
use wildmatch::WildMatchPattern;

#[derive(Debug, Clone)]
pub struct PostProcessor {
    pub name: String,
    pub bin: PathBuf,

    command: Arc<Mutex<Command>>
}

impl PostProcessor {
    pub fn new<N: Into<String>, B: Into<PathBuf>>(name: N, bin: B) -> Self {
        let name: String = name.into();
        let bin: PathBuf = bin.into();
        let mut command = Command::new(bin.clone());
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let command = Arc::new(Mutex::new(command));
        Self {
            name,
            bin,
            command
        }
    }

    pub fn new_stdin_stdout<N: Into<String>, B: Into<PathBuf>>(name: N, bin: B) -> Self {
        Self::new(name, bin)
    }

    pub fn cmd(&self) -> MutexGuard<Command> {
        self.command.lock().expect("Couldn't lock on command (arc)")
    }

    pub fn is_available(&self) -> bool {
        match Command::new(self.bin.clone()).spawn() {
            Ok(mut v) => {
                let _ = v.kill();
                true
            },
            Err(_) => false
        }
    }

    pub fn execute(&self, code: String) -> String {

        if !self.is_available() { return code }

        let mut spawned = self.cmd().spawn().unwrap_or_else(|_| panic!("Couldn't spawn {}", self.name));
        
        // send to stdin
        {
            let stdin = spawned.stdin.as_mut().unwrap_or_else(|| panic!("Couldn't get {}'s stdin", self.name));
            let mut writer = BufWriter::new(stdin);
            writer.write_all(code.as_bytes()).unwrap_or_else(|_| panic!("Couldn't write to {}'s stdin", self.name));
            writer.flush().unwrap_or_else(|_| panic!("Couldn't flush {} stdin", self.name));
        }

        // read from stdout
        {
            let res = spawned.wait_with_output().unwrap_or_else(|_| panic!("Couldn't wait for {} to finish", self.name));
            String::from_utf8(res.stdout).unwrap_or_else(|_| panic!("{} returned a non-utf8 code in stdout", self.name))
        }
    }

    pub fn get_default() -> Vec<Self> {
        let mut postprocessors = HashMap::new();
        
        let shfmt = PostProcessor::new_stdin_stdout("shfmt", "/usr/bin/shfmt");
        shfmt.cmd().arg("-i").arg("4");
        shfmt.cmd().arg("-ln").arg("bash");
        postprocessors.insert("shfmt", shfmt);
        
        let bshchk = PostProcessor::new_stdin_stdout("bshchk", "/usr/bin/bshchk");
        bshchk.cmd().arg("--ignore-shebang");
        postprocessors.insert("bshchk", bshchk);

        postprocessors.values().cloned().collect_vec()
    }

    pub fn filter_default(filters: Vec<WildMatchPattern<'*', '?'>>) -> Vec<Self> {
        let default = Self::get_default();

        default
            .iter()
            .filter(|x| {
                filters.iter()  
                    .any(|xx| !xx.matches(&x.name))
            })
            .cloned()
            .collect_vec()
    }
}
