use std::process::Command;

fn main() {
    // See [https://doc.rust-lang.org/cargo/reference/build-scripts.html].
    build_helper::rerun_if_changed("src/tests");

    let git_hash = Command::new("git").arg("rev-parse").arg("HEAD").output().unwrap();
    let git_hash = String::from_utf8(git_hash.stdout).unwrap();
    
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
