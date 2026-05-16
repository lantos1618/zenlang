use super::*;

#[test]
fn type_parameter_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

identity<T: Json> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Value, "identity")
        .expect("value symbol");
    let entries = TypeParameterAbsenceValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT", "type parameter count"),
            AbsentMetadataEntry::new(true, "NAMES", "type parameter names"),
            AbsentMetadataEntry::new(true, "BOUNDS", "type parameter bounds"),
            AbsentMetadataEntry::new(true, "BOUND_REFS", "typed type parameter bound refs"),
        ]
    );
}

#[test]
fn type_parameter_absence_validation_uses_module_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E0269");
    assert_eq!(validation.name_code, "E0348");
    assert_eq!(validation.bound_code, "E0270");
    assert_eq!(validation.bound_ref_code, "E0373");
}

#[test]
fn type_parameter_absence_validation_uses_import_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E0285");
    assert_eq!(validation.name_code, "E0349");
    assert_eq!(validation.bound_code, "E0286");
    assert_eq!(validation.bound_ref_code, "E0364");
}

#[test]
fn type_parameter_absence_validation_uses_local_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E0253");
    assert_eq!(validation.name_code, "E0350");
    assert_eq!(validation.bound_code, "E0254");
    assert_eq!(validation.bound_ref_code, "E0382");
}

#[test]
fn type_parameter_absence_validation_uses_variant_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E0334");
    assert_eq!(validation.name_code, "E0351");
    assert_eq!(validation.bound_code, "E0335");
    assert_eq!(validation.bound_ref_code, "E0391");
}
