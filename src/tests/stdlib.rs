use crate::compiler::AmberCompiler;
use std::fs;
use std::io::Read;
use std::io::Write;
use tempfile::tempdir;

macro_rules! test_amber {
    ($code:expr, $result:expr) => {
        {
            match AmberCompiler::new($code.to_string(), None).test_eval() {
                Ok(result) => assert_eq!(result.trim(), $result),
                Err(err) => panic!("ERROR: {}", err.message.unwrap())
            }

        }
    };
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
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let file_path = temp_dir.path().join("test_file.txt");

    let mut file = fs::File::create(&file_path).expect("Failed to create temporary file");
    file.write_all(b"This is sample file.").expect("Failed to write to temporary file");

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

    test_amber!(code, "This is sample file.");

    fs::remove_file(&file_path).expect("Failed to delete temporary file");
}

#[test]
fn file_write() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let file_path = temp_dir.path().join("test_file.txt");

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

    fs::remove_file(&file_path).expect("Failed to delete temporary file");
}

#[test]
fn file_append() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let file_path = temp_dir.path().join("test_file.txt");

    // 初期ファイルを作成して書き込む
    let mut initial_file = fs::File::create(&file_path).expect("Failed to create temporary file");
    initial_file.write_all(b"Hello, Amber!\n").expect("Failed to write to temporary file");

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

    assert_eq!(file_content.trim(), "Hello, Amber!\nAppending this line.");

    fs::remove_file(&file_path).expect("Failed to delete temporary file");
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
            echo trim(\"  hello world  \")
        }
    ";
    test_amber!(code, "hello world")
}

// TODO: Write test code for the trim_left function.
//#[test]
//fn trim_left() {
//    let code = "
//        import * from \"std\"
//        main {
//            echo trim_left(\"  hello world  \")
//        }
//    ";
//    test_amber!(code, "hello world  ")
//}

// TODO: Write test code for the trim_right function.
//#[test]
//fn trim_right() {
//    let code = "
//        import * from \"std\"
//        main {
//            echo trim_right(\"  hello world  \")
//        }
//    ";
//    test_amber!(code, "  hello world")
//}

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

// TODO: Unable to validate changes in exit code.
#[test]
fn exit() {
    let code = "
        import * from \"std\"
        main {
            exit(0)
        }
    ";
    test_amber!(code, "")
}

#[test]
fn includes() {
    let code = "
        import * from \"std\"
        main {
            if includes([\"apple\", \"banana\", \"cherry\"], \"banana\") {
                echo \"Found\"
            } else {
                echo \"Not Found\"
            }
        }
    ";
    test_amber!(code, "Found")
}

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

