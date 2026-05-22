use super::*;

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
