use super::*;

#[test]
fn variant_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup_variant("Option", "Some")
        .expect("variant symbol");
    let entries = VariantAbsenceValidation {
        names_code: "NAMES",
        owner_code: "OWNER",
        payload_count_code: "PAYLOAD_COUNT",
        payload_type_name_code: "PAYLOAD_TYPE",
        payload_type_code: "TYPED_PAYLOAD",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(false, "NAMES", "variant names"),
            AbsentMetadataEntry::new(true, "OWNER", "variant owner"),
            AbsentMetadataEntry::new(true, "PAYLOAD_COUNT", "variant payload count"),
            AbsentMetadataEntry::new(true, "PAYLOAD_TYPE", "variant payload type"),
            AbsentMetadataEntry::new(true, "TYPED_PAYLOAD", "typed variant payload type"),
        ]
    );
}

#[test]
fn variant_absence_validation_uses_module_resolver_codes() {
    let validation = VariantAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.names_code, "E0273");
    assert_eq!(validation.owner_code, "E0274");
    assert_eq!(validation.payload_count_code, "E0275");
    assert_eq!(validation.payload_type_name_code, "E0276");
    assert_eq!(validation.payload_type_code, "E0375");
}

#[test]
fn variant_absence_validation_uses_import_resolver_codes() {
    let validation = VariantAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.names_code, "E0289");
    assert_eq!(validation.owner_code, "E0290");
    assert_eq!(validation.payload_count_code, "E0291");
    assert_eq!(validation.payload_type_name_code, "E0292");
    assert_eq!(validation.payload_type_code, "E0366");
}

#[test]
fn variant_absence_validation_uses_local_resolver_codes() {
    let validation = VariantAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.names_code, "E0257");
    assert_eq!(validation.owner_code, "E0258");
    assert_eq!(validation.payload_count_code, "E0259");
    assert_eq!(validation.payload_type_name_code, "E0260");
    assert_eq!(validation.payload_type_code, "E0384");
}

#[test]
fn variant_absence_validation_uses_type_like_resolver_codes() {
    let validation = VariantAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.names_code, "E0315");
    assert_eq!(validation.owner_code, "E0316");
    assert_eq!(validation.payload_count_code, "E0317");
    assert_eq!(validation.payload_type_name_code, "E0318");
    assert_eq!(validation.payload_type_code, "E0397");
}

#[test]
fn variant_absence_validation_uses_behavior_resolver_codes() {
    let validation = VariantAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.names_code, "E0323");
    assert_eq!(validation.owner_code, "E0324");
    assert_eq!(validation.payload_count_code, "E0325");
    assert_eq!(validation.payload_type_name_code, "E0326");
    assert_eq!(validation.payload_type_code, "E0400");
}

#[test]
fn variant_absence_validation_uses_value_resolver_codes() {
    let validation = VariantAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.names_code, "E0300");
    assert_eq!(validation.owner_code, "E0301");
    assert_eq!(validation.payload_count_code, "E0302");
    assert_eq!(validation.payload_type_name_code, "E0303");
    assert_eq!(validation.payload_type_code, "E0404");
}
