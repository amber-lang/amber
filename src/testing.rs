use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::TestCommand;
use colored::Colorize;
use heraclitus_compiler::prelude::Message;
use rayon::prelude::*;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn find_amber_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_amber_files(&path, files)?;
            } else if let Some(ext) = path.extension() {
                if ext == "ab" {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

pub fn get_tests_to_run(
    command: &TestCommand,
) -> Result<Vec<(PathBuf, String, String)>, Vec<Message>> {
    let input_path = &command.input;
    let mut files = vec![];
    if input_path.is_dir() {
        find_amber_files(input_path, &mut files)
            .map_err(|e| vec![Message::new_err_msg(e.to_string())])?;
    } else {
        files.push(input_path.clone());
    }
    files.sort();

    // Discovery phase
    let mut tests = vec![];
    let mut errors = vec![];
    for file in &files {
        let code = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                errors.push(Message::new_err_msg(format!(
                    "Failed to read file {}: {}",
                    file.display(),
                    e
                )));
                continue;
            }
        };

        let options = CompilerOptions::from_args(&command.no_proc, false, true, None);
        let compiler = AmberCompiler::new(
            code.clone(),
            Some(file.to_string_lossy().to_string()),
            options,
        );

        match compiler.tokenize() {
            Ok(tokens) => match compiler.parse(tokens) {
                Ok((_, meta)) => {
                    for name in meta.test_names {
                        tests.push((file.clone(), name, code.clone()));
                    }
                }
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            },
            Err(e) => {
                errors.push(e);
                continue;
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Filter tests
    if let Some(pattern) = command.args.first() {
        tests.retain(|(file, name, _)| {
            let test_name_display = if name.is_empty() {
                format!("{}", file.display())
            } else {
                format!("{} ({})", file.display(), name)
            };
            test_name_display.contains(pattern)
        });
    }

    Ok(tests)
}

pub fn handle_test(command: TestCommand) -> Result<i32, Box<dyn Error>> {
    let tests = match get_tests_to_run(&command) {
        Ok(t) => t,
        Err(errors) => {
            for e in errors {
                e.show();
            }
            return Ok(1);
        }
    };

    let total = tests.len();
    if total == 0 {
        println!("No tests found");
        return Ok(0);
    }

    let failed = std::sync::Mutex::new(vec![]);

    tests
        .par_iter()
        .enumerate()
        .for_each(|(i, (file, name, code))| {
            let test_name_display = if name.is_empty() {
                format!("{}", file.display())
            } else {
                format!("{} ({})", file.display(), name)
            };

            let options =
                CompilerOptions::from_args(&command.no_proc, false, true, Some(name.clone()));
            let compiler = AmberCompiler::new(
                code.clone(),
                Some(file.to_string_lossy().to_string()),
                options,
            );

            let result = match compiler.compile() {
                Ok((_, bash_code)) => {
                    match Command::new("bash")
                        .args(["--norc", "-c"])
                        .arg(&bash_code)
                        .output()
                    {
                        Ok(output) => {
                            if output.status.success() {
                                Ok(())
                            } else {
                                let err_msg = format!(
                                    "{}\n{}",
                                    String::from_utf8_lossy(&output.stdout),
                                    String::from_utf8_lossy(&output.stderr)
                                )
                                .trim()
                                .to_string();
                                if err_msg.is_empty() {
                                    Err(Message::new_err_msg("(No output)".dimmed().to_string()))
                                } else {
                                    Err(Message::new_err_msg(err_msg))
                                }
                            }
                        }
                        Err(e) => Err(Message::new_err_msg(format!("Error executing bash: {}", e))),
                    }
                }
                Err(e) => Err(e),
            };

            match result {
                Ok(_) => {
                    println!(
                        "[{}/{}] {} {} ... {}",
                        i + 1,
                        total,
                        "✓".green(),
                        test_name_display,
                        "Success".green()
                    );
                }
                Err(msg) => {
                    println!(
                        "[{}/{}] {} {} ... {}",
                        i + 1,
                        total,
                        "×".red(),
                        test_name_display,
                        "Failed".red()
                    );
                    failed.lock().unwrap().push((i + 1, test_name_display, msg));
                }
            }
        });

    let failed_vec = failed.lock().unwrap();
    if !failed_vec.is_empty() {
        println!();
        for (i, name, msg) in failed_vec.iter() {
            println!("[{i}] {} failed with:", name);
            if let Some(m) = &msg.message {
                println!("{}", m);
            } else {
                msg.show();
            }
            println!();
        }

        println!("{}", "Summary of failed tests:".red());
        for (_, name, _) in failed_vec.iter() {
            println!("{} {}", "×".red(), name);
        }
        println!();
    }

    let passed_count = total - failed_vec.len();
    let failed_count = failed_vec.len();

    if failed_count == 0 {
        println!("{}", " Success ".white().on_green());
    } else {
        println!("{}", " Failure ".white().on_red());
    }
    println!(
        "{} tests passed | {} tests failed",
        passed_count, failed_count
    );

    if failed_count > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}
