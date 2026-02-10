fn main() {
    // See [https://doc.rust-lang.org/cargo/reference/build-scripts.html].
    build_helper::rerun_if_changed("src/tests");
    built::write_built_file().expect("Failed to acquire build-time information");
}
