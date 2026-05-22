use super::*;

#[test]
fn v1_spec_is_single_source_of_truth_and_old_spec_is_quarantined() {
    let old_spec = read("LANGUAGE_SPEC.zen");
    let old_spec_header = old_spec.lines().take(20).collect::<Vec<_>>().join("\n");

    assert!(
        old_spec.lines().count() <= 80,
        "LANGUAGE_SPEC.zen is an archive pointer, not a second language spec"
    );
    assert!(
        old_spec_header.contains("Archived aspirational notes"),
        "LANGUAGE_SPEC.zen must be explicitly quarantined at the top"
    );
    assert!(
        old_spec_header.contains("docs/V1_SPEC.md is the versioned v1 source of truth"),
        "LANGUAGE_SPEC.zen must point to docs/V1_SPEC.md as the source of truth"
    );
    for stale_syntax in ["return ", "@std", ".implements(Geometric,"] {
        assert!(
            !old_spec.contains(stale_syntax),
            "LANGUAGE_SPEC.zen archive pointer should not preserve stale syntax examples: {stale_syntax}"
        );
    }
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

#[test]
fn live_compiler_and_stdlib_sources_do_not_claim_llvm_intrinsics() {
    let output = std::process::Command::new("git")
        .args(["ls-files", "src", "stdlib"])
        .current_dir(repo_root())
        .output()
        .expect("list live compiler and stdlib sources");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tracked = String::from_utf8(output.stdout).expect("git ls-files output is utf-8");
    for path in tracked
        .lines()
        .filter(|path| path.ends_with(".rs") || path.ends_with(".zen") || path.ends_with(".md"))
    {
        let source = std::fs::read_to_string(repo_root().join(path)).expect("read tracked file");
        for stale in ["LLVM IR", "Raw Rust/LLVM", "LLVM atomics"] {
            assert!(
                !source.contains(stale),
                "{path} should not claim live compiler or stdlib intrinsics are LLVM-specific: {stale}"
            );
        }
    }
}

#[test]
fn compiler_intrinsic_definitions_live_in_focused_helper() {
    let root = read("src/intrinsics.rs");
    let definitions = read("src/intrinsics/definitions.rs");

    for root_detail in [
        "macro_rules! intrinsic",
        "fn build_intrinsics",
        r#"intrinsic!(m, "raw_allocate""#,
        r#"intrinsic!(m, "atomic_load""#,
        r#"intrinsic!(m, "syscall6""#,
    ] {
        assert!(
            !root.contains(root_detail),
            "compiler intrinsic root should not own intrinsic definition-table detail: {root_detail}"
        );
        assert!(
            definitions.contains(root_detail),
            "compiler intrinsic definition table should live in focused helper: {root_detail}"
        );
    }

    assert!(
        root.contains("mod definitions;"),
        "compiler intrinsic root should include the focused definitions helper"
    );
    assert!(
        root.contains("definitions::build_intrinsics"),
        "compiler intrinsic root should initialize the registry through the focused helper"
    );
    assert!(
        root.lines().count() < 130,
        "compiler intrinsic root should stay focused on module recognition and public registry API"
    );
}

#[test]
fn stdlib_async_operation_state_lives_in_one_helper_module() {
    assert!(
        !repo_root()
            .join("stdlib/memory/async_allocator.zen")
            .exists(),
        "async operation state helpers should not masquerade as an allocator module"
    );

    let helpers = read("stdlib/memory/async_helpers.zen");
    for required in ["AsyncOp:", "Promise:", "async_op_new", "Promise.new"] {
        assert!(
            helpers.contains(required),
            "stdlib/memory/async_helpers.zen should own async operation state helper: {required}"
        );
    }
}
