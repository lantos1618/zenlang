use super::*;

#[test]
fn v1_spec_is_single_source_of_truth_and_old_spec_is_quarantined() {
    let old_spec = read("LANGUAGE_SPEC.zen");
    let old_spec_header = old_spec.lines().take(20).collect::<Vec<_>>().join("\n");

    assert!(
        old_spec_header.contains("Archived aspirational notes"),
        "LANGUAGE_SPEC.zen must be explicitly quarantined at the top"
    );
    assert!(
        old_spec_header.contains("docs/V1_SPEC.md is the versioned v1 source of truth"),
        "LANGUAGE_SPEC.zen must point to docs/V1_SPEC.md as the source of truth"
    );
}

#[test]
fn aspirational_stdlib_is_explicitly_experimental() {
    let stdlib_readme = read("stdlib/README.md");

    for required in [
        "experimental",
        "not part of the implemented v1 surface",
        "must parse, typecheck, and build",
        "docs/V1_SPEC.md",
    ] {
        assert!(
            stdlib_readme.contains(required),
            "stdlib/README.md is missing experimental gate text: {required}"
        );
    }
}

#[test]
fn stale_rewrite_audit_docs_are_removed() {
    for path in ["docs/REWRITE.md", "docs/AUDIT_PROMPT.md"] {
        assert!(
            !repo_root().join(path).exists(),
            "{path} is stale rewrite/audit scaffolding; keep durable status in PHASE_PLAN and COMPLETION_AUDIT"
        );
    }
}

#[test]
fn generated_vscode_packages_are_not_tracked() {
    let output = std::process::Command::new("git")
        .args(["ls-files", "*.vsix"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked VSIX packages");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tracked = String::from_utf8(output.stdout).expect("git ls-files output is utf-8");
    assert!(
        tracked.trim().is_empty(),
        "generated VS Code packages should not be checked in:\n{tracked}"
    );

    let gitignore = read(".gitignore");
    assert!(
        gitignore.lines().any(|line| line.trim() == "*.vsix"),
        ".gitignore should keep generated VSIX packages out of source control"
    );
}
