use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn prints_raw_template_without_creating_files_or_stripping_comments() {
    let dir = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_idr"))
        .arg("template")
        .current_dir(dir.path())
        .env("IDR_NO_COMMENTS", "1")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        include_bytes!("../src/default_template.md.jinja")
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
