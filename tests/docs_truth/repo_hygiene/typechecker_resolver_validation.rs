use super::*;

#[test]
fn typechecker_resolver_validation_post_pass_lives_in_focused_helper() {
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let post_pass = read("src/typechecker/resolver_validation/post_pass.rs");

    for helper in [
        "validate_no_extra_resolver_declaration_symbols",
        "validate_no_extra_resolver_local_symbols",
        "validate_resolver_behavior_association_lists",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver validation entry traversal should not own post-pass helper: {helper}"
        );
        assert!(
            post_pass.contains(&format!("fn {helper}")),
            "resolver validation post-pass helper should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/post_pass.rs\");"),
        "resolver validation should include focused post-pass validation"
    );
}

#[test]
fn typechecker_resolver_expected_value_symbols_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let monolith = read("src/typechecker/resolver_validation_support/expected_symbols.rs");
    let focused = read("src/typechecker/resolver_validation_support/expected_value_symbols.rs");

    for helper in [
        "ExpectedValueSignature",
        "ExpectedValueSymbol",
        "ExpectedParameter",
        "ExpectedReturnMetadata",
        "ValueParameterValidation",
        "ReturnValidation",
    ] {
        assert!(
            !monolith.contains(&format!("struct {helper}")),
            "expected_symbols.rs should not own value-symbol helper: {helper}"
        );
        assert!(
            focused.contains(&format!("struct {helper}")),
            "expected value-symbol helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation_support/expected_value_symbols.rs\");"),
        "resolver validation support should include focused expected value-symbol helpers"
    );
}

#[test]
fn typechecker_resolver_expected_formatting_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let helpers = read("src/typechecker/resolver_validation_support/expected_helpers.rs");
    let formatting = read("src/typechecker/resolver_validation_support/expected_formatting.rs");

    for helper in [
        "visibility_name",
        "mutability_name",
        "resolver_count_display",
        "resolver_metadata_display",
        "resolver_ast_type_metadata_display",
        "optional_ast_type_display",
        "format_type_parameter_names",
        "format_type_parameter_bounds",
        "format_type_parameter_bound_refs",
        "format_parameter_type_names",
        "format_ast_type_list",
        "format_parameter_names",
        "format_field_types",
        "format_field_type_names",
        "format_variant_names",
        "format_resolver_string_list",
        "format_resolver_display_list",
        "join_resolver_strings",
        "join_resolver_display_values",
        "format_resolver_named_list",
        "format_behavior_method_signatures",
        "format_behavior_method_types",
        "format_behavior_ref_names",
        "format_behavior_refs",
        "format_resolver_nonempty_joined_list",
        "behavior_ref_names_match",
        "behavior_refs_match",
    ] {
        assert!(
            !helpers.contains(&format!("fn {helper}")),
            "expected_helpers.rs should not own resolver formatting helper: {helper}"
        );
        assert!(
            formatting.contains(&format!("fn {helper}")),
            "resolver expected formatting should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation_support/expected_formatting.rs\");"),
        "resolver validation support should include focused expected formatting helper"
    );
}

#[test]
fn typechecker_resolver_pattern_local_traversal_lives_in_focused_helper() {
    let traversal = read("src/typechecker/resolver_validation/local_traversal.rs");
    let patterns = read("src/typechecker/resolver_validation/pattern_locals.rs");

    for helper in [
        "require_resolver_pattern_expr_locals",
        "require_resolver_pattern_locals",
        "require_resolver_pattern_binding",
    ] {
        assert!(
            !traversal.contains(&format!("fn {helper}")),
            "resolver local traversal should not own pattern-local helper: {helper}"
        );
        assert!(
            patterns.contains(&format!("fn {helper}")),
            "resolver pattern-local traversal should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/pattern_locals.rs\");"),
        "resolver validation should include focused pattern-local traversal"
    );
}

#[test]
fn typechecker_resolver_variant_metadata_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let types = read("src/typechecker/resolver_validation/metadata_types.rs");
    let variants = read("src/typechecker/resolver_validation/metadata_variants.rs");

    for helper in [
        "validate_resolver_variant_names",
        "validate_resolver_variant_payload",
        "validate_resolver_variant_owner_name",
        "validate_resolver_variant_visibility",
        "validate_resolver_variant_absent_other_metadata",
    ] {
        assert!(
            !types.contains(&format!("fn {helper}")),
            "resolver type metadata should not own variant metadata helper: {helper}"
        );
        assert!(
            variants.contains(&format!("fn {helper}")),
            "resolver variant metadata helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation/metadata_variants.rs\");"),
        "resolver validation should include focused variant metadata helper"
    );
}

#[test]
fn typechecker_resolver_absence_diagnostics_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let diagnostics = read("src/typechecker/resolver_validation/metadata_diagnostics.rs");
    let absence = read("src/typechecker/resolver_validation/metadata_absence.rs");

    for helper in [
        "validate_resolver_absent_value_signature_metadata",
        "validate_resolver_absent_type_parameter_metadata",
        "validate_resolver_absent_field_metadata",
        "validate_resolver_absent_variant_metadata",
        "validate_resolver_absent_behavior_association_metadata",
        "validate_resolver_absent_behavior_declaration_metadata",
        "validate_resolver_absent_mutability_metadata",
        "validate_resolver_absent_source_metadata",
    ] {
        assert!(
            !diagnostics.contains(&format!("fn {helper}")),
            "resolver metadata diagnostics should not own absence wrapper: {helper}"
        );
        assert!(
            absence.contains(&format!("fn {helper}")),
            "resolver absence diagnostics should live in focused helper: {helper}"
        );
    }

    assert!(
        diagnostics.lines().count() < 220,
        "resolver metadata diagnostics should stay focused on generic emitters"
    );
    assert!(
        root.contains("include!(\"resolver_validation/metadata_absence.rs\");"),
        "resolver validation should include focused absence diagnostics"
    );
}

#[test]
fn typechecker_resolver_type_behavior_metadata_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_type_behavior_metadata.rs");
    let type_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols.rs");
    let behavior_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/behavior_symbols.rs");

    for test_name in [
        "check_program_with_symbols_validates_resolver_type_parameter_counts",
        "check_program_with_symbols_validates_resolver_type_parameter_names",
        "check_program_with_symbols_validates_resolver_type_visibility",
        "check_program_with_symbols_validates_resolver_type_parameter_bounds",
        "check_program_with_symbols_validates_resolver_type_like_absent_value_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_type_behavior_metadata.rs should not own type metadata test: {test_name}"
        );
        assert!(
            type_metadata.contains(&format!("fn {test_name}")),
            "type metadata tests should live in focused module: {test_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_validates_resolver_behavior_visibility",
        "check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds",
        "check_program_with_symbols_validates_resolver_behavior_absent_type_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_type_behavior_metadata.rs should not own behavior metadata test: {test_name}"
        );
        assert!(
            behavior_metadata.contains(&format!("fn {test_name}")),
            "behavior metadata tests should live in focused module: {test_name}"
        );
    }

    for module_name in ["type_symbols", "behavior_symbols"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver type/behavior metadata root should include focused module: {module_name}"
        );
    }
}

#[test]
fn typechecker_resolver_declaration_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_declarations.rs");
    let symbols = read("src/typechecker/tests/resolver_declarations/symbols.rs");
    let imports = read("src/typechecker/tests/resolver_declarations/imports.rs");
    let methods = read("src/typechecker/tests/resolver_declarations/methods.rs");

    for test_name in [
        "check_program_with_symbols_requires_resolver_declarations",
        "check_program_with_symbols_rejects_extra_resolver_declarations",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own symbol declaration test: {test_name}"
        );
        assert!(
            symbols.contains(&format!("fn {test_name}")),
            "resolver declaration symbol tests should live in focused module: {test_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present",
        "check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present",
        "check_program_with_symbols_uses_resolver_import_bindings",
        "check_program_with_symbols_validates_stripped_resolver_import_sources",
        "check_program_with_symbols_validates_stripped_resolver_import_visibility",
        "check_program_with_symbols_requires_stripped_resolver_import_modules",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own resolver import test: {test_name}"
        );
        assert!(
            imports.contains(&format!("fn {test_name}")),
            "resolver declaration import tests should live in focused module: {test_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_requires_resolver_method_receiver_type",
        "check_program_with_symbols_validates_resolver_method_signature",
        "check_program_with_symbols_validates_resolver_method_function_type_signature",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own resolver method test: {test_name}"
        );
        assert!(
            methods.contains(&format!("fn {test_name}")),
            "resolver declaration method tests should live in focused module: {test_name}"
        );
    }

    for module_name in ["symbols", "imports", "methods"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver declarations root should include focused module: {module_name}"
        );
    }
}
