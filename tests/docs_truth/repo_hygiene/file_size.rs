use super::*;

#[test]
fn production_rust_files_stay_below_cleanup_threshold() {
    const MAX_LINES: usize = 400;

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

#[test]
fn declaration_validation_precollection_tasks_live_in_focused_helper() {
    let tasks = read("src/typechecker/tests/declaration_validation/tasks.rs");
    let precollection = read("src/typechecker/tests/declaration_validation/precollection_tasks.rs");
    let module = read("src/typechecker/tests/declaration_validation.rs");

    assert!(
        tasks.lines().count() < 240,
        "tasks.rs should stay focused on declaration semantic validation tasks"
    );
    assert!(
        !tasks.contains("self_type_context_validation_tasks_collect_declarations"),
        "precollection task tests should live in precollection_tasks.rs"
    );
    assert!(
        precollection.contains("self_type_context_validation_tasks_collect_declarations"),
        "precollection_tasks.rs should cover self type context task collection"
    );
    assert!(
        precollection.contains("ast_declaration_collection_bundle_replays_collection_passes"),
        "precollection_tasks.rs should cover declaration collection task replay"
    );
    assert!(
        module.contains("mod precollection_tasks;"),
        "declaration_validation.rs should include the focused precollection_tasks module"
    );
}

#[test]
fn struct_literal_default_tests_live_in_focused_helper() {
    let struct_literals = read("src/typechecker/tests/core_semantics/struct_literals.rs");
    let defaults = read("src/typechecker/tests/core_semantics/struct_literal_defaults.rs");
    let module = read("src/typechecker/tests/core_semantics.rs");

    assert!(
        struct_literals.lines().count() < 180,
        "struct_literals.rs should stay focused on struct literal error cases"
    );
    assert!(
        !struct_literals.contains("struct_literal_uses_default_for_omitted_field"),
        "struct literal default tests should live in struct_literal_defaults.rs"
    );
    assert!(
        defaults.contains("struct_literal_uses_default_for_omitted_field"),
        "struct_literal_defaults.rs should cover defaulted field omission"
    );
    assert!(
        defaults.contains("generic_struct_literal_uses_substituted_default_for_omitted_field"),
        "struct_literal_defaults.rs should cover generic default substitution"
    );
    assert!(
        module.contains("mod struct_literal_defaults;"),
        "core_semantics.rs should include the focused struct_literal_defaults module"
    );
}

#[test]
fn generic_behavior_impl_type_arg_tests_live_in_focused_helper() {
    let impls = read("src/typechecker/tests/generic_behaviors/impls_and_requires.rs");
    let type_args = read("src/typechecker/tests/generic_behaviors/impl_type_args.rs");
    let module = read("src/typechecker/tests/generic_behaviors.rs");

    assert!(
        impls.lines().count() < 180,
        "impls_and_requires.rs should stay focused on basic impl/require behavior tests"
    );
    assert!(
        !impls.contains("behavior_impl_generic_behavior_without_type_args_is_error"),
        "generic behavior impl type-argument tests should live in impl_type_args.rs"
    );
    assert!(
        type_args.contains("behavior_impl_generic_behavior_without_type_args_is_error"),
        "impl_type_args.rs should cover missing generic behavior type arguments"
    );
    assert!(
        type_args.contains("behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied"),
        "impl_type_args.rs should cover satisfied generic behavior type-argument bounds"
    );
    assert!(
        module.contains("mod impl_type_args;"),
        "generic_behaviors.rs should include the focused impl_type_args module"
    );
}

#[test]
fn lexer_string_interpolation_lives_in_focused_helper() {
    let strings = read("src/lexer/strings.rs");
    let interpolation = read("src/lexer/string_interpolation.rs");
    let lexer_module = read("src/lexer/mod.rs");

    assert!(
        strings.lines().count() < 160,
        "strings.rs should stay focused on literal string scanning"
    );
    assert!(
        !strings.contains("fn lex_interpolation_body"),
        "string interpolation body scanning should live in string_interpolation.rs"
    );
    assert!(
        interpolation.contains("fn lex_interpolation_body"),
        "string_interpolation.rs should scan interpolation bodies"
    );
    assert!(
        interpolation.contains("fn lex_next_no_skip"),
        "string_interpolation.rs should own no-skip token scanning for interpolation bodies"
    );
    assert!(
        lexer_module.contains("mod string_interpolation;"),
        "lexer module should include the focused string_interpolation helper"
    );
}

#[test]
fn lexer_number_scanning_lives_in_focused_helper() {
    let scan = read("src/lexer/scan.rs");
    let numbers = read("src/lexer/numbers.rs");
    let lexer_module = read("src/lexer/mod.rs");

    assert!(
        scan.lines().count() < 220,
        "scan.rs should stay focused on token dispatch and small token scanners"
    );
    assert!(
        !scan.contains("fn lex_prefixed_int"),
        "prefixed integer scanning should live in numbers.rs"
    );
    assert!(
        !scan.contains("fn eat_digits"),
        "digit scanning should live in numbers.rs"
    );
    assert!(
        numbers.contains("pub(super) fn lex_number"),
        "numbers.rs should own number token scanning"
    );
    assert!(
        numbers.contains("fn lex_prefixed_int"),
        "numbers.rs should own prefixed integer scanning"
    );
    assert!(
        lexer_module.contains("mod numbers;"),
        "lexer module should include the focused number scanning helper"
    );
}

#[test]
fn monomorphize_type_substitution_lives_in_focused_helper() {
    let monomorphize = read("src/typechecker/monomorphize.rs");
    let names = read("src/typechecker/monomorphize_names.rs");
    let substitution = read("src/typechecker/monomorphize_substitution.rs");
    let module = read("src/typechecker/mod.rs");

    assert!(
        monomorphize.lines().count() < 240,
        "monomorphize.rs should stay focused on callable specialization"
    );
    assert!(
        !monomorphize.contains("pub(crate) fn substitute_type"),
        "type substitution should live in monomorphize_substitution.rs"
    );
    assert!(
        names.contains("pub(crate) fn generic_function_mangled_name"),
        "monomorphize_names.rs should own generic callable mangling"
    );
    assert!(
        names.contains("pub(crate) fn mangle_generic_type_name"),
        "monomorphize_names.rs should own generic type mangling"
    );
    assert!(
        substitution.contains("pub(crate) fn substitute_type"),
        "monomorphize_substitution.rs should own generic AstType substitution"
    );
    assert!(
        module.contains("mod monomorphize_substitution;"),
        "typechecker module should include the focused monomorphize_substitution helper"
    );
    assert!(
        module.contains("mod monomorphize_names;"),
        "typechecker module should include the focused monomorphize_names helper"
    );
}
