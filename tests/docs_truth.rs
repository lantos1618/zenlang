use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    let path = repo_root().join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

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
        "zen emit-json ast <file>",
        "zen emit-json symbols <file>",
        "zen emit-json typed <file>",
        "zen emit-json diagnostics <file>",
        "build.zen",
        "Accepted Syntax Forms",
        "Test Evidence",
        "Planned Positive Test",
        "Planned Negative Test",
        "resolver_records_value_symbol_generic_parameter_counts",
        "resolver_records_value_symbol_function_type_metadata",
        "resolver_records_value_symbol_generic_bounds",
        "resolver_records_type_and_behavior_generic_parameter_counts",
        "resolver_rejects_duplicate_type_parameter_names",
        "resolver_records_generic_struct_field_types",
        "resolver_records_generic_enum_variant_payload_types",
        "resolver_records_generic_behavior_method_signatures",
        "resolver_records_generic_behavior_function_type_method_signatures",
        "parse_public_behavior_declaration",
        "resolver_records_public_visibility_for_exported_declarations",
        "resolver_rejects_duplicate_signature_parameter_names",
        "check_program_with_symbols_validates_resolver_function_type_parameter_metadata",
        "check_program_with_symbols_validates_resolver_function_type_return_metadata",
        "check_program_with_symbols_validates_resolver_function_typed_signature_metadata",
        "collect_declarations_with_symbols_uses_resolver_function_type_metadata",
        "collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata",
        "collect_declarations_with_symbols_uses_resolver_generic_method_template_metadata",
        "check_program_with_symbols_validates_resolver_function_type_parameter_names",
        "check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs",
        "check_program_with_symbols_validates_resolver_type_parameter_names",
        "behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied",
        "behavior_impl_generic_behavior_type_arg_bound_failure_is_error",
        "behavior_extends_generic_parent_accepts_child_type_parameter_arg",
        "check_program_with_symbols_validates_resolver_generic_struct_field_types",
        "check_program_with_symbols_validates_resolver_generic_enum_payload_types",
        "check_program_with_symbols_validates_resolver_generic_behavior_method_signatures",
        "check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures",
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
        "tests/zen/multi_file_generic_imported_type_dependency/main.zen",
        "tests/zen/multi_file_generic_imported_transitive_dependency/main.zen",
        "tests/zen/multi_file_type_method_worklist/main.zen",
        "tests/zen/multi_file_type_method_method_dependency/main.zen",
        "tests/zen/multi_file_type_method_imported_dependency/main.zen",
        "tests/zen/multi_file_type_impl_imported_type_dependency/main.zen",
        "tests/zen/multi_file_behavior_bound/main.zen",
        "tests/zen/multi_file_behavior_inheritance/main.zen",
        "tests/zen/multi_file_imported_behavior_impl/main.zen",
        "tests/zen/multi_file_imported_behavior_default/main.zen",
        "tests/zen/multi_file_imported_impl_imported_behavior/main.zen",
        "tests/zen/multi_file_imported_child_parent_dispatch/main.zen",
        "tests/zen/multi_file_imported_behavior_requires/main.zen",
        "tests/zen/multi_file_imported_function_imported_behavior_bound/main.zen",
        "tests/zen/multi_file_imported_function_param_type_dependency/main.zen",
        "tests/zen/multi_file_imported_function_return_type_dependency/main.zen",
        "tests/zen/multi_file_imported_function_imported_return_type_behavior/main.zen",
        "tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen",
        "generated_c_call_definition_scan_reports_missing_generated_calls",
        "generated_c_definition_count_ignores_prototypes",
        "imported_function_signature_type_dependencies_are_not_directly_visible",
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
        "imported_private_behavior_impl_methods_are_not_directly_visible",
        "resolver_rejects_duplicate_behavior_method_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_impl_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_required_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_parent_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_impl_refs",
        "check_program_with_symbols_validates_resolver_generic_behavior_required_refs",
        "check_program_with_symbols_validates_resolver_generic_behavior_parent_refs",
        "collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata",
        "collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata",
        "resolver_records_method_signatures_as_value_symbols",
        "resolver_records_method_function_type_signatures",
        "check_program_with_symbols_validates_resolver_method_signature",
        "check_program_with_symbols_validates_resolver_method_function_type_signature",
        "check_module_graph_entry_does_not_seed_private_methods_for_imported_types",
        "tests/zen/multi_file_type_method/main.zen",
        "test_multi_file_type_method_worklist_imports",
        "test_multi_file_type_method_method_dependency_imports",
        "test_multi_file_type_method_imported_dependency_imports",
        "imported_type_method_worklist_helpers_are_not_directly_visible",
        "imported_type_method_dependencies_are_not_directly_visible",
        "imported_type_method_imported_dependencies_are_not_directly_visible",
        "imported_type_impl_imported_type_dependencies_are_not_directly_visible",
        "imported_generic_function_imported_type_dependencies_are_not_directly_visible",
        "imported_generic_function_transitive_dependencies_are_not_directly_visible",
        "parse_impl_block",
        "resolver_accepts_non_behavior_impl_blocks_as_method_symbols",
        "resolver_rejects_duplicate_non_behavior_impl_method_names",
        "resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method",
        "tests/zen/type_impl_methods.zen",
        "tests/zen/multi_file_type_impl/main.zen",
        "imported_private_type_impl_methods_are_not_visible",
        "resolver_records_struct_function_type_fields",
        "check_program_with_symbols_validates_resolver_struct_function_type_fields",
        "check_program_with_symbols_validates_resolver_struct_typed_field_metadata",
        "collect_declarations_with_symbols_uses_resolver_struct_field_metadata",
        "resolver_records_enum_function_type_payloads",
        "check_program_with_symbols_validates_resolver_enum_function_type_payloads",
        "check_program_with_symbols_validates_resolver_enum_typed_payload_metadata",
        "collect_declarations_with_symbols_uses_resolver_enum_payload_metadata",
        "resolver_records_generic_enum_function_type_payloads",
        "check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads",
        "resolver_records_behavior_function_type_method_signatures",
        "check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures",
        "check_program_with_symbols_validates_resolver_behavior_method_types",
        "collect_declarations_with_symbols_uses_resolver_behavior_method_metadata",
        "resolver_records_generic_behavior_function_type_method_signatures",
        "check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures",
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
        "resolver_records_mutable_closure_parameter_locals",
        "check_program_with_symbols_validates_resolver_closure_parameter_mutability",
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
        "deterministic_build_graph_creates_one_executable_target",
        "build_graph_rejects_undeclared_host_effects",
        "parse_project_build_zen_example",
        "parse_shorthand_enum_variant_expr_and_pattern",
        "parsed_project_build_zen_lowers_to_executable_and_test_graph",
        "build_program_lowering_collects_test_target",
        "build_program_lowering_rejects_undeclared_env_reads",
        "emit_json_build_graph_outputs_project_build_graph",
        "emit_json_build_graph_rejects_undeclared_host_effects",
        "emit_json_build_graph_rejects_undeclared_host_effects_before_test_target_lowering",
        "build_graph_command_compiles_single_executable_target",
        "build_graph_command_rejects_missing_root_source",
        "build_command_routes_build_zen_through_deterministic_graph",
        "build_command_build_zen_compiles_multiple_executable_targets",
        "build_command_build_zen_rejects_undeclared_host_effects",
        "build_command_multi_target_build_zen_rejects_undeclared_host_effects",
        "check_command_validates_build_zen_graph",
        "check_command_build_zen_rejects_undeclared_host_effects",
        "emit_command_build_zen_outputs_target_c_source",
        "emit_command_build_zen_rejects_undeclared_host_effects",
        "direct_file_command_build_zen_routes_through_deterministic_graph",
        "direct_file_command_build_zen_compiles_multiple_executable_targets",
        "direct_file_command_build_zen_rejects_undeclared_host_effects",
        "build_program_lowering_collects_multiple_executable_targets",
        "legacy_emit_json_modes_reject_build_zen_with_graph_diagnostic",
        "collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata",
        "collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore",
        "resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass",
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
        "build.zen entrypoints are not complete",
        "Do not mark the objective complete",
        "build_command_routes_build_zen_through_deterministic_graph",
        "build_command_build_zen_compiles_multiple_executable_targets",
        "build_command_multi_target_build_zen_rejects_undeclared_host_effects",
        "check_command_validates_build_zen_graph",
        "emit_command_build_zen_outputs_target_c_source",
        "direct_file_command_build_zen_routes_through_deterministic_graph",
        "direct_file_command_build_zen_compiles_multiple_executable_targets",
        "build_program_lowering_collects_multiple_executable_targets",
        "parsed_project_build_zen_lowers_to_executable_and_test_graph",
        "build_program_lowering_collects_test_target",
        "emit_json_build_graph_rejects_undeclared_host_effects_before_test_target_lowering",
        "legacy_emit_json_modes_reject_build_zen_with_graph_diagnostic",
        "collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata",
        "collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore",
        "resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass",
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
    let extension_readme = read("vscode-extension/README.md");
    let extension_package = read("vscode-extension/package.json");
    let extension_source = read("vscode-extension/src/extension.ts");
    let setup_lsp = read("setup_lsp.sh");

    assert!(ci.contains("cargo fmt --check"));
    assert!(ci.contains("cargo clippy -- -D warnings"));
    assert!(ci.contains("cargo test --lib"));
    assert!(ci.contains("cargo test --tests"));
    assert!(
        ci.contains("types: [opened, reopened, ready_for_review]"),
        "CI pull_request triggers must avoid synchronize spam while still running when PRs leave draft"
    );
    assert!(
        !ci.contains("synchronize"),
        "CI workflow should not run on every draft PR synchronize push"
    );
    assert_eq!(
        ci.matches("github.event.pull_request.draft == false")
            .count(),
        3,
        "CI fmt, clippy, and test jobs must keep the draft-PR guard"
    );
    assert!(
        ci.contains("workflow_dispatch"),
        "CI must keep a manual dispatch path when draft PR pushes do not run checks"
    );

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
