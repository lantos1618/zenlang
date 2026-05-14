use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    let path = repo_root().join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

#[test]
fn readme_only_advertises_rewrite_baseline() {
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
    ] {
        assert!(
            !readme.contains(stale_claim),
            "README still advertises unsupported rewrite-baseline claim: {stale_claim}"
        );
    }

    for required in [
        "rewrite",
        "C backend",
        "cargo fmt --check",
        "cargo clippy -- -D warnings",
        "cargo test --lib",
        "cargo test --tests",
        "docs/V1_SPEC.md",
    ] {
        assert!(
            readme.contains(required),
            "README is missing required truthful baseline text: {required}"
        );
    }
}

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

#[test]
fn v1_spec_records_phase_one_feature_gates_and_test_backlog() {
    let spec = read("docs/V1_SPEC.md");

    for required in [
        "Status: v1 draft",
        "Feature Matrix",
        "implemented",
        "gated",
        "experimental",
        "removed",
        "Sync/Async effects",
        "Typed allocators",
        "Type matching",
        "Behavior association",
        "AST traversal",
        "Actors in std",
        "JSON/YAML IR boundaries",
        "build.zen",
        "Accepted Syntax Forms",
        "Test Evidence",
        "Planned Positive Test",
        "Planned Negative Test",
        "resolver_records_value_symbol_generic_parameter_counts",
        "resolver_records_value_symbol_function_type_metadata",
        "resolver_records_type_and_behavior_generic_parameter_counts",
        "resolver_rejects_duplicate_type_parameter_names",
        "resolver_records_generic_struct_field_types",
        "resolver_records_generic_enum_variant_payload_types",
        "resolver_records_generic_behavior_method_signatures",
        "parse_public_behavior_declaration",
        "resolver_records_public_visibility_for_exported_declarations",
        "resolver_rejects_duplicate_signature_parameter_names",
        "check_program_with_symbols_validates_resolver_function_type_parameter_metadata",
        "check_program_with_symbols_validates_resolver_function_type_return_metadata",
        "check_program_with_symbols_validates_resolver_function_type_parameter_names",
        "check_program_with_symbols_validates_resolver_type_parameter_names",
        "check_program_with_symbols_validates_resolver_generic_struct_field_types",
        "check_program_with_symbols_validates_resolver_generic_enum_payload_types",
        "check_program_with_symbols_validates_resolver_generic_behavior_method_signatures",
        "check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds",
        "resolver_rejects_duplicate_struct_field_names",
        "resolver_rejects_duplicate_struct_literal_fields",
        "resolver_rejects_unknown_struct_literal_fields",
        "resolver_rejects_missing_struct_literal_fields",
        "resolver_rejects_unknown_struct_literal_types",
        "resolver_allows_same_variant_names_in_different_enums",
        "resolver_rejects_duplicate_variant_names_in_same_enum",
        "resolver_rejects_unknown_enum_variant_expressions",
        "resolver_rejects_missing_enum_variant_payload_expressions",
        "resolver_rejects_unexpected_enum_variant_payload_expressions",
        "tests/zen/duplicate_enum_variant_names.zen",
        "check_module_graph_entry_seeds_imported_function_type_signatures",
        "check_module_graph_entry_specializes_imported_generic_functions",
        "check_module_graph_entry_specializes_imported_generic_enums",
        "check_module_graph_entry_seeds_public_methods_for_imported_types",
        "check_module_graph_entry_specializes_public_generic_methods_for_imported_types",
        "tests/zen/multi_file_generic/main.zen",
        "tests/zen/multi_file_behavior_bound/main.zen",
        "tests/zen/multi_file_behavior_inheritance/main.zen",
        "tests/zen/multi_file_imported_behavior_impl/main.zen",
        "tests/zen/multi_file_imported_behavior_default/main.zen",
        "tests/zen/multi_file_imported_impl_imported_behavior/main.zen",
        "tests/zen/multi_file_imported_child_parent_dispatch/main.zen",
        "tests/zen/multi_file_imported_behavior_requires/main.zen",
        "imported_behavior_extends_requires_parent_methods",
        "imported_behavior_extends_imported_parent_requires_parent_methods",
        "imported_behavior_extends_requires_transitive_parent_methods",
        "resolver_rejects_duplicate_behavior_impl_edges",
        "resolver_rejects_duplicate_behavior_required_edges",
        "resolver_rejects_duplicate_behavior_parent_edges",
        "tests/zen/generic_method_worklist.zen",
        "resolver_records_behavior_impl_methods_as_value_symbols",
        "resolver_records_behavior_impl_function_type_methods",
        "check_program_with_symbols_validates_resolver_impl_method_signature",
        "check_program_with_symbols_validates_resolver_impl_function_type_signature",
        "resolver_rejects_duplicate_behavior_method_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_impl_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_required_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_parent_names",
        "resolver_records_method_signatures_as_value_symbols",
        "resolver_records_method_function_type_signatures",
        "check_program_with_symbols_validates_resolver_method_signature",
        "check_program_with_symbols_validates_resolver_method_function_type_signature",
        "check_module_graph_entry_does_not_seed_private_methods_for_imported_types",
        "tests/zen/multi_file_type_method/main.zen",
        "parse_impl_block",
        "resolver_accepts_non_behavior_impl_blocks_as_method_symbols",
        "resolver_rejects_duplicate_non_behavior_impl_method_names",
        "resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method",
        "tests/zen/type_impl_methods.zen",
        "tests/zen/multi_file_type_impl/main.zen",
        "imported_private_type_impl_methods_are_not_visible",
        "resolver_records_struct_function_type_fields",
        "check_program_with_symbols_validates_resolver_struct_function_type_fields",
        "resolver_records_enum_function_type_payloads",
        "check_program_with_symbols_validates_resolver_enum_function_type_payloads",
        "resolver_records_behavior_function_type_method_signatures",
        "check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures",
        "resolver_records_behavior_impl_method_body_locals",
        "check_program_with_symbols_requires_resolver_impl_method_body_locals",
        "resolver_records_behavior_default_method_body_locals",
        "check_program_with_symbols_requires_resolver_behavior_default_locals",
        "resolver_records_struct_field_default_locals",
        "check_program_with_symbols_requires_resolver_struct_field_default_locals",
        "resolver_records_top_level_expr_locals",
        "check_program_with_symbols_requires_resolver_top_level_expr_locals",
        "resolver_records_closure_locals",
        "check_program_with_symbols_requires_resolver_closure_locals",
        "resolver_records_pattern_locals",
        "check_program_with_symbols_requires_resolver_pattern_locals",
        "resolver_records_same_name_locals_in_distinct_scopes",
        "check_program_with_symbols_validates_resolver_local_mutability_by_scope",
    ] {
        assert!(
            spec.contains(required),
            "docs/V1_SPEC.md is missing Phase 1 requirement: {required}"
        );
    }
}

#[test]
fn phase_plan_records_recovered_progress_and_next_slice() {
    let plan = read("docs/PHASE_PLAN.md");

    for required in [
        "Recovery Point",
        "183d140c",
        "Completed Evidence",
        "Current Phase",
        "Next Small Slice",
        "Sync/Async are real effects",
        "typed allocators",
        "actors live in std first",
        "AST/HIR traversal is tooling/metaprogramming",
        "type matching and behavior association are separate",
        "JSON is compiler-owned IR output",
        "YAML is human-authored config/spec input",
        "build.zen is deterministic comptime build graph",
    ] {
        assert!(
            plan.contains(required),
            "docs/PHASE_PLAN.md is missing durable plan text: {required}"
        );
    }

    assert!(
        !plan.contains("Generic behavior inheritance in `.extends` is still explicitly gated"),
        "docs/PHASE_PLAN.md still claims generic behavior inheritance is gated"
    );
}

#[test]
fn completion_audit_records_recovery_objective_evidence_and_gaps() {
    let audit = read("docs/COMPLETION_AUDIT.md");

    for required in [
        "Objective Restatement",
        "Prompt-To-Artifact Checklist",
        "183d140c",
        "docs/PHASE_PLAN.md",
        "cargo fmt --check",
        "cargo clippy -- -D warnings",
        "cargo test --lib",
        "cargo test --tests",
        "Design Decisions Preserved",
        "Unresolved Gaps",
        "Phase 4 is not complete",
        "Do not mark the objective complete",
    ] {
        assert!(
            audit.contains(required),
            "docs/COMPLETION_AUDIT.md is missing audit evidence text: {required}"
        );
    }

    assert!(
        !audit.contains("Generic behavior inheritance in `.extends` is still explicitly gated"),
        "docs/COMPLETION_AUDIT.md still claims generic behavior inheritance is gated"
    );
}

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
fn ci_and_release_only_advertise_existing_targets() {
    let ci = read(".github/workflows/ci.yml");
    let release = read(".github/workflows/release.yml");
    let makefile = read("Makefile");
    let vscode_settings = read(".vscode/settings.json");
    let vscode_launch = read(".vscode/launch.json");
    let agent = read(".claude/agents/zen-dev.yaml");
    let extension_readme = read("vscode-extension/README.md");
    let extension_package = read("vscode-extension/package.json");
    let extension_source = read("vscode-extension/src/extension.ts");
    let setup_lsp = read("setup_lsp.sh");

    assert!(ci.contains("cargo fmt --check"));
    assert!(ci.contains("cargo clippy -- -D warnings"));
    assert!(ci.contains("cargo test --lib"));
    assert!(ci.contains("cargo test --tests"));

    for unsupported in ["LLVM", "zen-lsp", "aarch64-apple-darwin"] {
        assert!(
            !release.contains(unsupported),
            "release workflow advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !makefile.contains(unsupported),
            "Makefile advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !vscode_settings.contains(unsupported),
            ".vscode/settings.json advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !vscode_launch.contains(unsupported),
            ".vscode/launch.json advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !agent.contains(unsupported),
            ".claude/agents/zen-dev.yaml advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !extension_readme.contains(unsupported),
            "vscode-extension/README.md advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !extension_package.contains(unsupported),
            "vscode-extension/package.json advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !extension_source.contains(unsupported),
            "vscode-extension/src/extension.ts advertises unsupported target/artifact: {unsupported}"
        );
        assert!(
            !setup_lsp.contains(unsupported),
            "setup_lsp.sh advertises unsupported target/artifact: {unsupported}"
        );
    }
}

#[test]
fn checked_in_configs_do_not_contain_secret_literals() {
    let config_paths = [
        ".github/copilot-mcp.json",
        ".github/workflows/ci.yml",
        ".github/workflows/release.yml",
        ".vscode/settings.json",
        ".vscode/launch.json",
        ".vscode/zen-language-config.json",
        ".claude/agents/zen-dev.yaml",
        ".claude/agents/zen-reviewer.yaml",
    ];

    for path in config_paths {
        let contents = read(path);
        for marker in ["Bearer ", "sk-", "ghp_", "github_pat_", "nk_"] {
            assert!(
                !contents.contains(marker),
                "{path} contains credential-looking marker {marker}"
            );
        }
    }
}
