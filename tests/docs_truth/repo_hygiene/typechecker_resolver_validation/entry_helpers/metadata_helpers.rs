use super::*;

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
fn typechecker_resolver_import_absence_metadata_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let symbols_locals = read("src/typechecker/resolver_validation/symbols_locals.rs");
    let import_absence = read("src/typechecker/resolver_validation/import_absence.rs");

    assert!(
        symbols_locals.lines().count() < 190,
        "resolver symbol/local validation should stay focused on lookup and role validation"
    );
    assert!(
        !symbols_locals.contains("fn validate_resolver_import_absent_declaration_metadata"),
        "resolver symbol/local validation should not own import absence metadata"
    );
    assert!(
        import_absence.contains("fn validate_resolver_import_absent_declaration_metadata"),
        "import absence metadata validation should live in focused helper"
    );

    for required in [
        "ValueSignatureAbsenceValidation::import_resolver_codes()",
        "TypeParameterAbsenceValidation::import_resolver_codes()",
        "FieldAbsenceValidation::import_resolver_codes()",
        "VariantAbsenceValidation::import_resolver_codes()",
        "BehaviorAssociationAbsenceValidation::import_resolver_codes()",
        "BehaviorDeclarationAbsenceValidation::import_resolver_codes()",
        "MutabilityAbsenceValidation::import_resolver_code()",
    ] {
        assert!(
            import_absence.contains(required),
            "import absence helper should keep resolver absence evidence: {required}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation/import_absence.rs\");"),
        "resolver validation should include focused import absence metadata helper"
    );
}
