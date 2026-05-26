use super::*;

#[test]
fn type_parameter_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
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
        count_code: "COUNT".into(),
        name_code: "NAMES".into(),
        bound_code: "BOUNDS".into(),
        bound_ref_code: "BOUND_REFS".into(),
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT".into(), "type parameter count"),
            AbsentMetadataEntry::new(true, "NAMES".into(), "type parameter names"),
            AbsentMetadataEntry::new(true, "BOUNDS".into(), "type parameter bounds"),
            AbsentMetadataEntry::new(true, "BOUND_REFS".into(), "typed type parameter bound refs"),
        ]
    );
}

#[test]
fn type_parameter_absence_validation_uses_module_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E7269");
    assert_eq!(validation.name_code, "E7348");
    assert_eq!(validation.bound_code, "E7270");
    assert_eq!(validation.bound_ref_code, "E7373");
}

#[test]
fn type_parameter_absence_validation_uses_import_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E7285");
    assert_eq!(validation.name_code, "E7349");
    assert_eq!(validation.bound_code, "E7286");
    assert_eq!(validation.bound_ref_code, "E7364");
}

#[test]
fn type_parameter_absence_validation_uses_local_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E7253");
    assert_eq!(validation.name_code, "E7350");
    assert_eq!(validation.bound_code, "E7254");
    assert_eq!(validation.bound_ref_code, "E7382");
}

#[test]
fn type_parameter_absence_validation_uses_variant_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E7334");
    assert_eq!(validation.name_code, "E7351");
    assert_eq!(validation.bound_code, "E7335");
    assert_eq!(validation.bound_ref_code, "E7391");
}
