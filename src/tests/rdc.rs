use std::process::{Command, Stdio};

use crate::compiler::AmberCompiler;

#[test]
fn nonexistant_command() {
    let compiler = AmberCompiler::new("unsafe $rfdhyikjeldrhfnjdkfmgdfk$".to_string(), None, false);
    let compiled = compiler.compile().map_or_else(Err, |(_, code)| {
        let child = Command::new("/bin/bash")
            .arg("-c")
            .arg(code.to_string())
            .stderr(Stdio::piped())
            .spawn().unwrap().wait_with_output().unwrap();
        assert_eq!(child.status.code(), Some(1));
        assert_eq!(String::from_utf8(child.stderr).unwrap(), "This program requires for these commands: ( rfdhyikjeldrhfnjdkfmgdfk ) to be present in $PATH.\n");
        Ok(())
    });
    compiled.unwrap();
}

#[test]
fn existant_command() {
    let compiler = AmberCompiler::new("unsafe $bash -c 'echo ok'$".to_string(), None, false);
    let compiled = compiler.compile().map_or_else(Err, |(_, code)| {
        let child = Command::new("/bin/bash")
            .arg("-c")
            .arg(code.to_string())
            .stdout(Stdio::piped())
            .spawn().unwrap().wait_with_output().unwrap();
        assert_eq!(child.status.code(), Some(0));
        assert_eq!(String::from_utf8(child.stdout).unwrap(), "ok\n");
        Ok(())
    });
    compiled.unwrap();
}