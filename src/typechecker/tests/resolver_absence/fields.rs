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
        count_code: "COUNT",
        type_name_code: "FIELD_TYPES",
        typed_code: "TYPED_FIELDS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT", "field count"),
            AbsentMetadataEntry::new(true, "FIELD_TYPES", "field types"),
            AbsentMetadataEntry::new(true, "TYPED_FIELDS", "typed field types"),
        ]
    );
}

#[test]
fn field_absence_validation_uses_module_resolver_codes() {
    let validation = FieldAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E0271");
    assert_eq!(validation.type_name_code, "E0272");
    assert_eq!(validation.typed_code, "E0374");
}

#[test]
fn field_absence_validation_uses_import_resolver_codes() {
    let validation = FieldAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E0287");
    assert_eq!(validation.type_name_code, "E0288");
    assert_eq!(validation.typed_code, "E0365");
}

#[test]
fn field_absence_validation_uses_local_resolver_codes() {
    let validation = FieldAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E0255");
    assert_eq!(validation.type_name_code, "E0256");
    assert_eq!(validation.typed_code, "E0383");
}

#[test]
fn field_absence_validation_uses_type_like_resolver_codes() {
    let validation = FieldAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.count_code, "E0319");
    assert_eq!(validation.type_name_code, "E0320");
    assert_eq!(validation.typed_code, "E0398");
}

#[test]
fn field_absence_validation_uses_variant_resolver_codes() {
    let validation = FieldAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E0336");
    assert_eq!(validation.type_name_code, "E0337");
    assert_eq!(validation.typed_code, "E0392");
}

#[test]
fn field_absence_validation_uses_behavior_resolver_codes() {
    let validation = FieldAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.count_code, "E0321");
    assert_eq!(validation.type_name_code, "E0322");
    assert_eq!(validation.typed_code, "E0399");
}

#[test]
fn field_absence_validation_uses_value_resolver_codes() {
    let validation = FieldAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.count_code, "E0298");
    assert_eq!(validation.type_name_code, "E0299");
    assert_eq!(validation.typed_code, "E0403");
}
