use crate::compiler::AmberCompiler;
use crate::test_amber;
use crate::tests::compile_code;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use tempfile::TempDir;
use std::process::{Command, Stdio};

fn mkfile() -> (PathBuf, TempDir) {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    assert!(temp_dir.path().is_dir(), "Temp directory is not a directory!");

    let file_path = temp_dir.path().join("test_file.txt");

    let mut file = fs::File::create(&file_path).expect("Failed to create temporary file");
    file.write_all(b"This is a sample file.\n").expect("Failed to write to temporary file");
    file.flush().expect("Failed to flush file");

    (file_path, temp_dir)
}

#[test]
fn input() {
    let prompt_message = "Please enter your name:";
    let code = format!(r#"
        import * from "std"
        main {{
            let name = input("{}")
            echo "Hello, " + name
        }}
    "#, prompt_message);

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
fn replace_once() {
    let code = "
        import * from \"std\"
        main {
            echo replace_once(\"hello world!\", \"world\", \"Amber\")
        }
    ";

    test_amber!(code, "hello Amber!")
}

#[test]
fn replace() {
    let code = "
        import * from \"std\"
        main {
            echo replace(\"banana banana\", \"banana\", \"apple\")
        }
    ";
    test_amber!(code, "apple apple")
}

#[test]
fn replace_regex() {
    let code = "
        import * from \"std\"
        main {
            echo replace_regex(\"abc123def\", \"[0-9][0-9]*\", \"456\")
        }
    ";
    test_amber!(code, "abc456def")
}

#[test]
fn file_read() {
    let (file_path, temp_dir) = mkfile();

    let code = format!(
        "
        import * from \"std\"
        main {{
            let f = file_read(\"{file_path}\") failed {{ echo \"Failed\" }}
            echo f
        }}
        ",
        file_path = file_path.to_str().unwrap()
    );

    test_amber!(code, "This is a sample file.");

    temp_dir.close().expect("Couldn't close temp dir");
}

#[test]
fn file_write() {
    let (file_path, temp_dir) = mkfile();

    let code = format!(
        "
        import * from \"std\"
        main {{
            unsafe file_write(\"{file_path}\", \"Hello, Amber!\")
        }}
        ",
        file_path = file_path.to_str().unwrap()
    );

    test_amber!(code, "");

    let mut file_content = String::new();
    fs::File::open(&file_path)
        .expect("Failed to open temporary file")
        .read_to_string(&mut file_content)
        .expect("Failed to read from temporary file");

    assert_eq!(file_content.trim(), "Hello, Amber!");

    temp_dir.close().expect("Couldn't close temp dir");
}

#[test]
fn file_append() {
    let (file_path, temp_dir) = mkfile();

    let code = format!(
        "
        import * from \"std\"
        main {{
            unsafe file_append(\"{file_path}\", \"Appending this line.\")
        }}
        ",
        file_path = file_path.to_str().unwrap()
    );

    test_amber!(code, "");

    let mut file_content = String::new();
    fs::File::open(&file_path)
        .expect("Failed to open temporary file")
        .read_to_string(&mut file_content)
        .expect("Failed to read from temporary file");

    assert_eq!(file_content.trim(), "This is a sample file.\nAppending this line.");

    temp_dir.close().expect("Couldn't close temp dir");
}

#[test]
fn split() {
    let code = "
        import * from \"std\"
        main {
            let array = split(\"apple,banana,cherry\", \",\")
            echo array[1]
        }
    ";
    test_amber!(code, "banana")
}

#[test]
fn join() {
    let code = "
        import * from \"std\"
        main {
            echo join([\"apple\", \"banana\", \"cherry\"], \", \")
        }
    ";
    test_amber!(code, "apple,banana,cherry")
}

#[test]
fn trim() {
    let code = "
        import * from \"std\"
        main {
            echo trim(\"  hello   world  \")
        }
    ";
    test_amber!(code, "hello   world")
}

#[test]
fn trim_left() {
    let code = "
        import * from \"std\"
        main {
            echo trim_left(\"  hello   world  \")
        }
    ";
    test_amber!(code, "hello   world  ")
}

#[test]
fn trim_right() {
    let code = "
        import * from \"std\"
        main {
            echo trim_right(\"  hello   world  \")
        }
    ";
    test_amber!(code, "  hello   world")
}

#[test]
fn lower() {
    let code = "
        import * from \"std\"
        main {
            echo lower(\"HELLO WORLD\")
        }
    ";
    test_amber!(code, "hello world")
}

#[test]
fn upper() {
    let code = "
        import * from \"std\"
        main {
            echo upper(\"hello world\")
        }
    ";
    test_amber!(code, "HELLO WORLD")
}

#[test]
fn len_string() {
    let code = "
        import * from \"std\"
        main {
            echo len(\"hello\")
        }
    ";
    test_amber!(code, "5")
}

#[test]
fn len_list() {
    let code = "
        import * from \"std\"
        main {
            echo len([1, 2, 3, 4])
        }
    ";
    test_amber!(code, "4")
}

#[test]
fn parse() {
    let code = "
        import * from \"std\"
        main {
            echo parse(\"123\")?
        }
    ";
    test_amber!(code, "123")
}

#[test]
fn chars() {
    let code = "
        import * from \"std\"
        main {
            echo chars(\"hello\")
        }
    ";
    test_amber!(code, "h e l l o")
}

#[test]
fn sum() {
    let code = "
        import * from \"std\"
        main {
            echo sum([1, 2, 3, 4])
        }
    ";
    test_amber!(code, "10")
}

#[test]
fn has_failed() {
    let code = "
        import * from \"std\"
        main {
            if has_failed(\"ls /nonexistent\") {
                echo \"Command failed\"
            } else {
                echo \"Command succeeded\"
            }
        }
    ";
    test_amber!(code, "Command failed")
}

#[test]
fn exit() {
    let code = "
        import * from \"std\"
        main {
            exit(37)
        }
    ";
    let code = compile_code(code);
    let mut cmd = Command::new("bash")
        .arg("-c").arg(code)
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn().expect("Couldn't spawn bash");

    assert_eq!(cmd.wait().expect("Couldn't wait for bash to execute").code(), Some(37));
}

macro_rules! test_includes {
    ($name:ident, $array_declaration:expr, $element:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let array_declaration = $array_declaration.to_string();
            let element = $element.to_string();
            let code = format!(r#"
                import * from "std"

                main {{
                    let array = {array_declaration}
                    if includes(array, {element}) {{
                        echo "Found"
                    }} else {{
                        echo "Not Found"
                    }}
                }}
            "#);

            test_amber!(code, $expected.to_string())
        }
    }
}

test_includes!(includes_empty_text_array, r#"[Text]"#, "\"\"", "Not Found");
test_includes!(includes_text_array, r#"["apple", "banana", "cherry"]"#, "\"banana\"", "Found");
test_includes!(includes_exact_match, r#"["apple", "banana cherry"]"#, "\"banana cherry\"", "Found");
test_includes!(includes_prefix_match, r#"["apple", "banana cherry"]"#, "\"banana\"", "Not Found");
test_includes!(includes_partial_match_with_expanded_element, r#"["foo", "bar", "baz"]"#, "\"oo ba\"", "Not Found");
test_includes!(includes_empty_num_array, r#"[Num]"#, 0, "Not Found");

#[test]
fn dir_exist() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let code = format!(
        "
        import * from \"std\"
        main {{
            if dir_exist(\"{tmpdir}\") {{
                echo \"Found\"
            }} else {{
                echo \"Not Found\"
            }}
        }}
        ",
        tmpdir = temp_dir.path().to_str().unwrap()
    );
    test_amber!(code, "Found")
}

#[test]
fn file_exist() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let file_path = temp_dir.path().join("test_file.txt");

    let _file = fs::File::create(&file_path).expect("Failed to create temporary file");

    let code = format!(
        "
        import * from \"std\"
        main {{
            if file_exist(\"{file_path}\") {{
                echo \"Found\"
            }} else {{
                echo \"Not Found\"
            }}
        }}
        ",
        file_path = file_path.to_str().unwrap()
    );
    test_amber!(code, "Found");

    fs::remove_file(&file_path).expect("Failed to delete temporary file");
}

#[test]
fn lines() {
    let code = "
        import { lines } from \"std\"
        main {
            loop line in lines(\"hello\\nworld\") {
                echo \"line: \" + line
            }
        }
    ";
    test_amber!(code, "line: hello\nline: world")
}
