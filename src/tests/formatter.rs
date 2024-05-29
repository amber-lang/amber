use std::{env, fs::{self, Permissions}, os::unix::fs::PermissionsExt};

use crate::modules::formatter::BashFormatter;

fn create_fake_binary(fmt: BashFormatter) {
    let body = if cfg!(unix) {
        "#!/usr/bin/env bash\nexit 0"
    } else {
        panic!("this test is not available for non-unix platforms")
    };

    let name: String = fmt.as_cmd();
    
    fs::write(&name, body).expect("Couldn't write fake script");
    fs::set_permissions(&name, Permissions::from_mode(0o755)).expect("Couldn't set perms for fake script");
}

#[test]
fn all_exist() {
    let path = env::var("PATH").expect("Cannot get $PATH");

    env::set_var("PATH", format!("{path}:./")); // temporary unset to ensure that shfmt exists in $PATH
    let fmts = BashFormatter::get_all();
    for fmt in fmts {
        create_fake_binary(fmt);
        assert_eq!(fmt.is_available(), true);
        assert_eq!(BashFormatter::get_available().is_some(), true);
        fs::remove_file(fmt.as_cmd::<String>()).expect("Couldn't remove formatter's fake binary");
    }

    env::set_var("PATH", &path);
    assert_eq!(env::var("PATH").expect("Cannot get $PATH"), path);
}
