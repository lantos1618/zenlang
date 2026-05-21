use super::super::*;

#[test]
fn resolver_behavior_ref_validation_descriptor_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let behavior_refs = read("src/typechecker/resolver_validation_support/behavior_refs.rs");
    let validation = read("src/typechecker/resolver_validation_support/behavior_ref_validation.rs");

    for helper in [
        "BehaviorRefValidation",
        "BehaviorRefRole",
        "BehaviorRefCheck",
    ] {
        assert!(
            !behavior_refs.contains(&format!("struct {helper}"))
                && !behavior_refs.contains(&format!("enum {helper}")),
            "behavior_refs.rs should not own validation descriptor helper: {helper}"
        );
        assert!(
            validation.contains(&format!("struct {helper}"))
                || validation.contains(&format!("enum {helper}")),
            "behavior-ref validation descriptor helper should live in focused helper: {helper}"
        );
    }

    assert!(
        behavior_refs.lines().count() < 180,
        "behavior_refs.rs should stay focused on expected behavior-edge metadata"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/behavior_ref_validation.rs\");"),
        "resolver validation support should include focused behavior-ref validation helper"
    );
}

#[test]
fn resolver_behavior_absence_descriptors_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let mixed = read("src/typechecker/resolver_validation_support/absence_symbol_descriptors.rs");
    let behavior =
        read("src/typechecker/resolver_validation_support/behavior_absence_descriptors.rs");

    for helper in [
        "BehaviorAssociationAbsenceValidation",
        "BehaviorDeclarationAbsenceValidation",
    ] {
        assert!(
            !mixed.contains(&format!("struct {helper}")),
            "absence_symbol_descriptors.rs should not own behavior absence descriptor: {helper}"
        );
        assert!(
            behavior.contains(&format!("struct {helper}")),
            "behavior absence descriptor should live in focused helper: {helper}"
        );
    }

    assert!(
        mixed.lines().count() < 140,
        "absence_symbol_descriptors.rs should stay focused on non-behavior descriptors"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/behavior_absence_descriptors.rs\");"),
        "resolver validation support should include focused behavior absence descriptors"
    );
}

#[test]
fn expected_local_traversal_support_stays_split_by_responsibility() {
    let root = read("src/typechecker/resolver_validation_support/expected_local_traversal.rs");
    let expressions =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/expressions.rs");
    let statements =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/statements.rs");
    let patterns =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/patterns.rs");
    let bindings =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/bindings.rs");

    assert!(
        root.lines().count() < 80,
        "expected_local_traversal.rs should only include focused traversal helpers"
    );
    for include in [
        "include!(\"expected_local_traversal/bindings.rs\");",
        "include!(\"expected_local_traversal/expressions.rs\");",
        "include!(\"expected_local_traversal/patterns.rs\");",
        "include!(\"expected_local_traversal/statements.rs\");",
    ] {
        assert!(
            root.contains(include),
            "expected local traversal root should include focused helper: {include}"
        );
    }
    assert!(
        !root.contains("fn expected_resolver_statement_locals"),
        "expected local traversal root should not own statement traversal bodies"
    );
    assert!(
        expressions.contains("fn expected_resolver_expr_locals"),
        "expressions.rs should cover expression traversal"
    );
    assert!(
        statements.contains("fn expected_resolver_statement_locals"),
        "statements.rs should cover statement traversal"
    );
    assert!(
        patterns.contains("fn expected_resolver_pattern_locals"),
        "patterns.rs should cover pattern traversal"
    );
    assert!(
        bindings.contains("fn expected_resolver_local"),
        "bindings.rs should cover shared local binding helpers"
    );
}

#[test]
fn resolver_backed_type_info_constructors_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support/type_info_constructors.rs");
    let resolver_backed = read(
        "src/typechecker/resolver_validation_support/type_info_constructors/resolver_backed.rs",
    );

    assert!(
        root.lines().count() < 180,
        "type_info_constructors.rs should stay focused on AST-backed constructors"
    );
    assert!(
        root.contains("include!(\"type_info_constructors/resolver_backed.rs\");"),
        "type info constructors should include focused resolver-backed constructors"
    );

    for helper in [
        "behavior_info_for_resolver_backed_stub",
        "struct_info_from_resolver_fields",
        "enum_info_from_resolver_variants",
        "behavior_info_from_resolver_methods",
        "func_info_from_behavior_method",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "type_info_constructors.rs should not own resolver-backed helper: {helper}"
        );
        assert!(
            resolver_backed.contains(&format!("fn {helper}")),
            "resolver-backed type info constructor should live in resolver_backed.rs: {helper}"
        );
    }
}

#[test]
fn imported_method_signature_support_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let field_variant = read("src/typechecker/resolver_validation_support/field_variant_scope.rs");
    let imported_signature =
        read("src/typechecker/resolver_validation_support/imported_method_signature.rs");

    for helper in [
        "ImportedMethodSignature",
        "from_function_declaration",
        "from_method_declaration",
        "func_info",
        "generic_template",
    ] {
        assert!(
            !field_variant.contains(helper),
            "field_variant_scope.rs should not own imported method signature helper: {helper}"
        );
        assert!(
            imported_signature.contains(helper),
            "imported method signature helper should live in focused helper: {helper}"
        );
    }

    assert!(
        field_variant.lines().count() < 170,
        "field_variant_scope.rs should stay focused on field and variant metadata"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/imported_method_signature.rs\");"),
        "resolver validation support should include focused imported method signature helper"
    );
}
