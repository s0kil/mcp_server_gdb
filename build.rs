use std::path::Path;
use std::process::Command;

fn main() {
    let fixtures_dir = Path::new("tests/fixtures");
    if !fixtures_dir.exists() {
        return;
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let test_program_src = fixtures_dir.join("test_program.c");
    if test_program_src.exists() {
        let status = Command::new("gcc")
            .args(["-g", "-O0", "-o"])
            .arg(format!("{}/test_program", out_dir))
            .arg(&test_program_src)
            .status()
            .expect("Failed to compile test_program.c - is gcc installed?");
        assert!(status.success(), "gcc failed to compile test_program.c");
        println!("cargo::rerun-if-changed=tests/fixtures/test_program.c");
    }

    let multi_thread_src = fixtures_dir.join("multi_thread.c");
    if multi_thread_src.exists() {
        let status = Command::new("gcc")
            .args(["-g", "-O0", "-pthread", "-o"])
            .arg(format!("{}/multi_thread", out_dir))
            .arg(&multi_thread_src)
            .status()
            .expect("Failed to compile multi_thread.c - is gcc installed?");
        assert!(status.success(), "gcc failed to compile multi_thread.c");
        println!("cargo::rerun-if-changed=tests/fixtures/multi_thread.c");
    }
}
