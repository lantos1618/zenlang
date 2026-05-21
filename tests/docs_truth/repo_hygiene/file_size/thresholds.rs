use super::super::*;

#[test]
fn production_rust_files_stay_below_cleanup_threshold() {
    const MAX_LINES: usize = 270;

    let output = std::process::Command::new("git")
        .args(["ls-files", "*.rs"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked Rust files");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let paths = String::from_utf8(output.stdout).expect("git ls-files output is utf-8");
    assert!(!paths.trim().is_empty(), "expected tracked Rust files");

    for path in paths.lines() {
        if !repo_root().join(path).exists() {
            continue;
        }
        let line_count = read(path).lines().count();
        assert!(
            line_count <= MAX_LINES,
            "{path} has {line_count} lines; split focused helpers before growing past {MAX_LINES}"
        );
    }
}

#[test]
fn zen_source_files_stay_below_cleanup_threshold() {
    const MAX_LINES: usize = 600;

    let output = std::process::Command::new("git")
        .args(["ls-files", "*.zen"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked Zen files");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let paths = String::from_utf8(output.stdout).expect("git ls-files output is utf-8");
    assert!(!paths.trim().is_empty(), "expected tracked Zen files");

    for path in paths.lines().filter(|path| {
        path.starts_with("examples/") || path.starts_with("stdlib/") || path.starts_with("tests/")
    }) {
        if !repo_root().join(path).exists() {
            continue;
        }
        let line_count = read(path).lines().count();
        assert!(
            line_count < MAX_LINES,
            "{path} has {line_count} lines; split focused helpers or remove generated scaffolding before growing to {MAX_LINES}+"
        );
    }
}
