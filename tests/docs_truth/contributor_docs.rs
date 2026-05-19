use super::*;

#[test]
fn contributor_docs_require_tests_first_for_language_work() {
    let contributing = read("CONTRIBUTING.md");

    for required in [
        "Failing tests first",
        "parser",
        "semantic",
        "effects",
        "stdlib",
        "codegen",
        "tooling",
        "C backend",
    ] {
        assert!(
            contributing.contains(required),
            "CONTRIBUTING.md is missing TDD/baseline requirement: {required}"
        );
    }

    for stale_claim in [
        "LLVM code generation",
        "src/lsp",
        "cargo build --bin zen-lsp",
    ] {
        assert!(
            !contributing.contains(stale_claim),
            "CONTRIBUTING.md still documents unsupported rewrite-baseline workflow: {stale_claim}"
        );
    }
}
