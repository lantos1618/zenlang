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
        names_code: "NAMES".into(),
        owner_code: "OWNER".into(),
        payload_count_code: "PAYLOAD_COUNT".into(),
        payload_type_name_code: "PAYLOAD_TYPE".into(),
        payload_type_code: "TYPED_PAYLOAD".into(),
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(false, "NAMES".into(), "variant names"),
            AbsentMetadataEntry::new(true, "OWNER".into(), "variant owner"),
            AbsentMetadataEntry::new(true, "PAYLOAD_COUNT".into(), "variant payload count"),
            AbsentMetadataEntry::new(true, "PAYLOAD_TYPE".into(), "variant payload type"),
            AbsentMetadataEntry::new(true, "TYPED_PAYLOAD".into(), "typed variant payload type"),
        ]
    );
}

#[test]
fn variant_absence_validation_uses_module_resolver_codes() {
    let validation = VariantAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.names_code, "E7273");
    assert_eq!(validation.owner_code, "E7274");
    assert_eq!(validation.payload_count_code, "E7275");
    assert_eq!(validation.payload_type_name_code, "E7276");
    assert_eq!(validation.payload_type_code, "E7375");
}

#[test]
fn variant_absence_validation_uses_import_resolver_codes() {
    let validation = VariantAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.names_code, "E7289");
    assert_eq!(validation.owner_code, "E7290");
    assert_eq!(validation.payload_count_code, "E7291");
    assert_eq!(validation.payload_type_name_code, "E7292");
    assert_eq!(validation.payload_type_code, "E7366");
}

#[test]
fn variant_absence_validation_uses_local_resolver_codes() {
    let validation = VariantAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.names_code, "E7257");
    assert_eq!(validation.owner_code, "E7258");
    assert_eq!(validation.payload_count_code, "E7259");
    assert_eq!(validation.payload_type_name_code, "E7260");
    assert_eq!(validation.payload_type_code, "E7384");
}

#[test]
fn variant_absence_validation_uses_type_like_resolver_codes() {
    let validation = VariantAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.names_code, "E7315");
    assert_eq!(validation.owner_code, "E7316");
    assert_eq!(validation.payload_count_code, "E7317");
    assert_eq!(validation.payload_type_name_code, "E7318");
    assert_eq!(validation.payload_type_code, "E7397");
}

#[test]
fn variant_absence_validation_uses_behavior_resolver_codes() {
    let validation = VariantAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.names_code, "E7323");
    assert_eq!(validation.owner_code, "E7324");
    assert_eq!(validation.payload_count_code, "E7325");
    assert_eq!(validation.payload_type_name_code, "E7326");
    assert_eq!(validation.payload_type_code, "E7400");
}

#[test]
fn variant_absence_validation_uses_value_resolver_codes() {
    let validation = VariantAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.names_code, "E7300");
    assert_eq!(validation.owner_code, "E7301");
    assert_eq!(validation.payload_count_code, "E7302");
    assert_eq!(validation.payload_type_name_code, "E7303");
    assert_eq!(validation.payload_type_code, "E7404");
}
