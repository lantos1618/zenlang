use super::*;

#[test]
fn production_rust_files_stay_below_cleanup_threshold() {
    const MAX_LINES: usize = 500;

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

#[test]
fn resolver_metadata_queue_selection_tests_live_in_focused_helper() {
    let helper = read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers.rs");
    let queue_helper = read("src/typechecker/tests/resolver_metadata/queue_selection.rs");
    let module = read("src/typechecker/tests/resolver_metadata.rs");

    assert!(
        helper.lines().count() < 260,
        "impl_and_method_helpers.rs should stay focused on impl/method metadata helpers"
    );
    assert!(
        !helper.contains("named_queue_selection_prefers_exact_then_front"),
        "queue-selection tests should live in queue_selection.rs"
    );
    assert!(
        queue_helper.contains("resolver_behavior_ref_queue_selection_prefers_exact_then_front"),
        "queue_selection.rs should cover behavior ref queue selection"
    );
    assert!(
        queue_helper.contains("named_queue_selection_can_preserve_front_for_future_match"),
        "queue_selection.rs should cover future-front preservation"
    );
    assert!(
        module.contains("mod queue_selection;"),
        "resolver_metadata.rs should include the focused queue_selection module"
    );
}
