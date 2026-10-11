use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn idr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_idr"))
}

#[test]
fn requires_explicit_paths() {
    let output = idr().arg("lint").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("<FILES>"));
}

#[test]
fn checks_multiple_files_without_modifying_or_scanning_others() {
    let dir = tempdir().unwrap();
    let draft = dir.path().join("draft.md");
    let final_idr = dir.path().join("final.md");
    let text = "---\ndraft: false\n---\n<!-- remove -->\n## Working notes\n";
    fs::write(&draft, text).unwrap();
    fs::write(&final_idr, "# Decision\n\nWe chose this for a reason.\n").unwrap();

    let output = idr()
        .arg("lint")
        .args([&final_idr, &draft])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 3);
    for (line, code) in [(2, "IDR001"), (4, "IDR002"), (5, "IDR003")] {
        assert!(stdout.contains(&format!("{}:{line}: {code} ", draft.display())));
    }
    assert_eq!(fs::read_to_string(&draft).unwrap(), text);

    let output = idr()
        .current_dir(dir.path())
        .args(["lint", "final.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn errors_do_not_skip_remaining_files_or_get_overwritten_by_findings() {
    let dir = tempdir().unwrap();
    let bad = dir.path().join("bad.md");
    let missing = dir.path().join("missing.md");
    let draft = dir.path().join("draft.md");
    fs::write(&bad, "---\ndraft: [\n---\n").unwrap();
    fs::write(&draft, "## Working notes\n").unwrap();
    let output = idr()
        .arg("lint")
        .args([&bad, &missing, &draft])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("IDR003"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&bad.display().to_string()));
    assert!(stderr.contains(&missing.display().to_string()));
    assert!(stderr.contains("Invalid YAML frontmatter"));
}

#[test]
fn generated_drafts_fail_until_manually_finalized() {
    for no_comments in [false, true] {
        let dir = tempdir().unwrap();
        let mut command = idr();
        command
            .current_dir(dir.path())
            .args(["new", "Example"])
            .env_remove("IDR_NO_COMMENTS")
            .env("GIT_AUTHOR_NAME", "Test Author")
            .env("GIT_AUTHOR_EMAIL", "test@example.com");
        if no_comments {
            command.arg("--no-comments");
        }
        assert!(command.output().unwrap().status.success());
        let path = fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("---\ndraft: true\n---\n"));

        let output = idr().arg("lint").arg(&path).output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("IDR001"));
        assert_eq!(stdout.contains("IDR002"), !no_comments);
        assert!(stdout.contains("IDR003"));

        // Simulate editorial cleanup, retaining a rationale and other metadata.
        fs::write(&path, "---\nowner: Test Author\n---\n# Example\n\n## Overview\nWe chose this to make releases easier.\n").unwrap();
        let output = idr().arg("lint").arg(&path).output().unwrap();
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
    }
}
