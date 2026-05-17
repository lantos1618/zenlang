use super::*;

#[test]
fn readme_is_language_first_and_links_status_docs() {
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
        "docs/learn_zen_in_y_minutes.md",
        "examples/README.md",
        "docs/V1_SPEC.md",
        "docs/PHASE_PLAN.md",
        "docs/COMPLETION_AUDIT.md",
    ] {
        assert!(
            readme.contains(required),
            "README is missing required language or docs pointer text: {required}"
        );
    }
}

#[test]
fn examples_index_uses_canonical_tutorial_and_project_paths() {
    let examples = read("examples/README.md");

    for required in [
        "docs/learn_zen_in_y_minutes.md",
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/project/main.zen",
    ] {
        assert!(
            examples.contains(required),
            "examples/README.md is missing canonical path: {required}"
        );
    }

    for stale_path in [
        "examples/hello_world.zen",
        "examples/variables_and_types.zen",
        "examples/pattern_matching.zen",
        "examples/structs_and_methods.zen",
        "examples/loops_and_closures.zen",
        "examples/error_handling.zen",
        "examples/demo_project",
    ] {
        assert!(
            !examples.contains(stale_path),
            "examples/README.md still references redundant path: {stale_path}"
        );
    }
}

#[test]
fn learn_zen_guide_covers_core_tour_and_gated_previews() {
    let guide = read("docs/learn_zen_in_y_minutes.md");

    for required in [
        "## Loops",
        "loop",
        "l.done()",
        "l.next()",
        "done(l)",
        "next(l)",
        "## Imports And Modules",
        "## Defer",
        "## Gated Preview: Sync, Async, And Allocators",
        "Sync",
        "Async",
        "Allocator<T, Sync>",
        "Allocator<T, Async>",
        "gated design",
        "docs/V1_SPEC.md",
        "examples/05_loops.zen",
    ] {
        assert!(
            guide.contains(required),
            "Learn guide is missing expected tour or gated-preview text: {required}"
        );
    }
}

#[test]
fn public_language_docs_and_examples_do_not_teach_return_keyword() {
    for path in [
        "README.md",
        "docs/learn_zen_in_y_minutes.md",
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/ffi_demo.zen",
        "examples/project/main.zen",
        "examples/project/math_utils.zen",
        "examples/project/build.zen",
        "examples/unified_allocator_demo.zen",
        "tests/nested_struct_field_access.zen",
    ] {
        let contents = read(path);
        assert!(
            !contents.contains("return "),
            "{path} still teaches the removed return keyword"
        );
    }
}

#[test]
fn stale_generated_tooling_directories_are_removed() {
    for path in [".claude", "scripts", "examples/demo_project"] {
        let absolute_path = repo_root().join(path);
        assert!(
            !absolute_path.exists(),
            "{path} should not exist; stale generated tooling/examples should stay out of the repo"
        );
    }
}
