use std::{path::PathBuf, sync::mpsc::{channel, Sender}};

use include_dir::{include_dir, Dir, File};
use indicatif::{ProgressBar, ProgressStyle};

use crate::compiler::{file_source::{FileMeta, FileSource}, AmberCompiler, CompilerOptions};
pub const STDLIB: Dir = include_dir!("src/std");

pub fn resolve<T: Into<String>>(path: T) -> Option<String> {
    let path = path.into();

    if let Some(module) = STDLIB.get_file(path + ".ab") {
        let module = module.contents_utf8().unwrap().to_string();
        Some(module)
    } else {
        None
    }
}

pub fn precompile_all() {
    let mut threads = vec![];
    let (tx, rx) = channel::<String>();

    fn precompile_thread(file: File, tx: Sender<String>) {
        let prefix: PathBuf = "std/".into();
        let mut path = prefix.join(file.path());
        path.set_extension("");
        let path = path.to_str().unwrap().to_string();

        let compiler = AmberCompiler::new(
            file.contents_utf8().unwrap().into(),
            Some(path.clone()),
            CompilerOptions {
                no_cache: false,
                no_proc: vec![String::from("*")],
                minify: false
            },
            FileMeta {
                is_import: true,
                source: FileSource::Stdlib
            }
        );

        compiler.tokenize().unwrap();

        tx.send(path).unwrap();
    }

    for file in STDLIB.files() {
        let tx = tx.clone();

        threads.push(std::thread::spawn(move || {
            let caught = std::panic::catch_unwind(|| {
                precompile_thread(file.clone(), tx.clone());
            });
            if caught.is_err() {
                tx.send("error".into()).unwrap()
            }
        }));
    }

    let mut i = 0;
    let progress = ProgressBar::new(threads.len() as u64)
        .with_style(ProgressStyle::with_template("{progress} [{elapsed}] {bar} {wide_msg}").unwrap());

    while let Ok(file) = rx.recv() {
        progress.inc(1);
        progress.set_message(file);

        i += 1;
        if i == threads.len() { break }
    }

    progress.finish_with_message("done");
}
