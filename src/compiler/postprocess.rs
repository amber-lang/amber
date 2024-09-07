use std::{collections::HashMap, io::{BufWriter, Write}, path::PathBuf, process::{Command, Stdio}, sync::{Arc, Mutex, MutexGuard}};

use itertools::Itertools;
use wildmatch::WildMatchPattern;

use crate::Cli;

/// How it will pass bash code to the postprocessor
#[derive(Debug, Clone)]
pub enum PostProcessorInput {
    /// Passes the data to the postprocessor's stdin
    Stdin
}

/// How it will get the processed code from the postprocessor
#[derive(Debug, Clone)]
pub enum PostProcessorOutput {
    /// Reads postprocessor's stdout
    Stdout
}

pub trait PostProcessorCommandModifier {
    /// Apply the command modifier to a command
    fn apply(&self, cmd: &mut MutexGuard<Command>);
}

impl PostProcessorCommandModifier for PostProcessorInput {
    fn apply(&self, cmd: &mut MutexGuard<Command>) {
        match self {
            Self::Stdin => cmd.stdin(Stdio::piped())
        };
    }
}

impl PostProcessorCommandModifier for PostProcessorOutput {
    fn apply(&self, cmd: &mut MutexGuard<Command>) {
        match self {
            Self::Stdout => cmd.stdout(Stdio::piped())
        };
    }
}

impl Default for PostProcessorInput {
    fn default() -> Self {
        Self::Stdin
    }
}

impl Default for PostProcessorOutput {
    fn default() -> Self {
        Self::Stdout
    }
}

#[derive(Debug, Clone)]
pub struct PostProcessor {
    pub name: String,
    pub bin: PathBuf,
    pub input: PostProcessorInput,
    pub output: PostProcessorOutput,

    command: Arc<Mutex<Command>>
}

impl PostProcessor {
    pub fn new<N: Into<String>, B: Into<PathBuf>>(name: N, bin: B, input: PostProcessorInput, output: PostProcessorOutput) -> Self {
        let name: String = name.into();
        let bin: PathBuf = bin.into();
        let command = Command::new(bin.clone());
        let command = Arc::new(Mutex::new(command));
        let thiss = Self { name, bin, input, output, command };
        thiss.build_cmd();
        thiss
    }

    pub fn new_stdin_stdout<N: Into<String>, B: Into<PathBuf>>(name: N, bin: B) -> Self {
        Self::new(name, bin, PostProcessorInput::default(), PostProcessorOutput::default())
    }

    pub fn cmd(&self) -> MutexGuard<Command> {
        self.command.lock().expect("Couldn't lock on command (arc)")
    }

    fn build_cmd(&self) {
        let mut command = self.cmd();
        self.input.apply(&mut command);
        self.output.apply(&mut command);
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
        
        match self.input {
            PostProcessorInput::Stdin => {
                let stdin = spawned.stdin.as_mut().unwrap_or_else(|| panic!("Couldn't get {}'s stdin", self.name));
                let mut writer = BufWriter::new(stdin);
                writer.write_all(code.as_bytes()).unwrap_or_else(|_| panic!("Couldn't write to {}'s stdin", self.name));
                writer.flush().unwrap_or_else(|_| panic!("Couldn't flush {} stdin", self.name));
            }
        };

        match self.output {
            PostProcessorOutput::Stdout => {
                let res = spawned.wait_with_output().unwrap_or_else(|_| panic!("Couldn't wait for {} to finish", self.name));
                String::from_utf8(res.stdout).unwrap_or_else(|_| panic!("{} returned a non-utf8 code in stdout", self.name))
            }
        }
    }

    pub fn get_default(cli: Cli) -> Vec<Self> {
        let mut postprocessors = HashMap::new();
        
        let shfmt = PostProcessor::new_stdin_stdout("shfmt", "/usr/bin/shfmt");
        shfmt.cmd().arg("-i").arg("4");
        shfmt.cmd().arg("-ln").arg("bash");
        postprocessors.insert("shfmt", shfmt);
        
        let bshchk = PostProcessor::new_stdin_stdout("bshchk", "/usr/bin/bshchk");
        bshchk.cmd().arg("--ignore-shebang");
        postprocessors.insert("bshchk", bshchk);

        for postprocessor in cli.disable_postprocessor.iter() {
            postprocessors.remove(postprocessor.as_str());
        }

        postprocessors.values().cloned().collect_vec()
    }

    pub fn filter_default(default: Vec<Self>, filters: Vec<WildMatchPattern<'*', '?'>>) -> Vec<Self> {
        if filters.is_empty() {
            return default
        }

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
