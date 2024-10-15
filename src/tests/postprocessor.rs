use std::{
    env::current_dir, fs::{self, Permissions}, os::unix::fs::PermissionsExt
};

use crate::compiler::postprocess::PostProcessor;

fn create_fake_binary(processor: &PostProcessor) {
    let body = if cfg!(unix) {
        "#!/usr/bin/env bash\nexit 0"
    } else {
        panic!("this test is not available for non-unix platforms")
    };

    fs::write(&processor.bin, body).expect("Couldn't write fake script");
    fs::set_permissions(&processor.bin, Permissions::from_mode(0o755))
        .expect("Couldn't set perms for fake script");
}

#[test]
fn all_exist() {
    let processor = PostProcessor::new(
        "test",
        current_dir().unwrap().join("test.sh"),
    );

    create_fake_binary(&processor);
    
    assert!(processor.is_available(), "Postprocessor is unavailable but it should be! It is likely an issue with the environment");
    fs::remove_file("test.sh").expect("Couldn't remove fake script");
}
