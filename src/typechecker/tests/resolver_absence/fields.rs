use super::*;

#[test]
fn field_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let entries = FieldAbsenceValidation {
        count_code: "COUNT".into(),
        type_name_code: "FIELD_TYPES".into(),
        typed_code: "TYPED_FIELDS".into(),
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT".into(), "field count"),
            AbsentMetadataEntry::new(true, "FIELD_TYPES".into(), "field types"),
            AbsentMetadataEntry::new(true, "TYPED_FIELDS".into(), "typed field types"),
        ]
    );
}

#[test]
fn field_absence_validation_uses_module_resolver_codes() {
    let validation = FieldAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E7271");
    assert_eq!(validation.type_name_code, "E7272");
    assert_eq!(validation.typed_code, "E7374");
}

#[test]
fn field_absence_validation_uses_import_resolver_codes() {
    let validation = FieldAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E7287");
    assert_eq!(validation.type_name_code, "E7288");
    assert_eq!(validation.typed_code, "E7365");
}

#[test]
fn field_absence_validation_uses_local_resolver_codes() {
    let validation = FieldAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E7255");
    assert_eq!(validation.type_name_code, "E7256");
    assert_eq!(validation.typed_code, "E7383");
}

#[test]
fn field_absence_validation_uses_type_like_resolver_codes() {
    let validation = FieldAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.count_code, "E7319");
    assert_eq!(validation.type_name_code, "E7320");
    assert_eq!(validation.typed_code, "E7398");
}

#[test]
fn field_absence_validation_uses_variant_resolver_codes() {
    let validation = FieldAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E7336");
    assert_eq!(validation.type_name_code, "E7337");
    assert_eq!(validation.typed_code, "E7392");
}

#[test]
fn field_absence_validation_uses_behavior_resolver_codes() {
    let validation = FieldAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.count_code, "E7321");
    assert_eq!(validation.type_name_code, "E7322");
    assert_eq!(validation.typed_code, "E7399");
}

#[test]
fn field_absence_validation_uses_value_resolver_codes() {
    let validation = FieldAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.count_code, "E7298");
    assert_eq!(validation.type_name_code, "E7299");
    assert_eq!(validation.typed_code, "E7403");
}
