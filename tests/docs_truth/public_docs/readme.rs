use super::super::*;

#[test]
fn readme_is_language_first_and_links_reference_docs() {
    let readme = read("README.md");

    for stale_claim in [
        "Late Alpha",
        "90% Core Complete",
        "ZERO KEYWORDS",
        "Full IDE support",
        "LLVM 18",
        "zen-lsp",
        "examples/showcase.zen",
        "codegen/llvm",
        "work-in-progress systems language compiler",
        "Current Baseline",
        "Repository Layout",
        "Current implementation status",
        "status",
        "gates",
        "audit details live",
        "docs/PHASE_PLAN.md",
        "docs/COMPLETION_AUDIT.md",
        "cargo fmt --check",
        "cargo clippy -- -D warnings",
        "cargo test --lib",
        "cargo test --tests",
        "rewrite branch as the baseline",
        "not a complete v1 language",
    ] {
        assert!(
            !readme.contains(stale_claim),
            "README should stay language-focused and avoid status/dev workflow text: {stale_claim}"
        );
    }

    for required in [
        "Zen",
        "Prefix-first declarations",
        "pattern matching",
        "behaviors",
        "StaticString",
        "allocator-backed String",
        "docs/learn_zen_in_y_minutes.md",
        "examples/README.md",
        "docs/V1_SPEC.md",
    ] {
        assert!(
            readme.contains(required),
            "README is missing required language or docs pointer text: {required}"
        );
    }
}
