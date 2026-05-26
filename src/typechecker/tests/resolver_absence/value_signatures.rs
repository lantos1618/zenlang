use super::*;

#[test]
fn value_signature_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
add = (left: i32, right: i32) i32 { left + right }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Value, "add")
        .expect("value symbol");
    let entries = ValueSignatureAbsenceValidation {
        parameter_count_code: "PARAM_COUNT".into(),
        parameter_name_code: "PARAM_NAMES".into(),
        parameter_type_name_code: "PARAM_TYPES".into(),
        parameter_type_code: "TYPED_PARAM_TYPES".into(),
        return_type_code: "RETURN_TYPE".into(),
        typed_return_type_code: "TYPED_RETURN_TYPE".into(),
    }
    .entries(symbol);

    assert!(entries.iter().all(|entry| entry.present));
    assert_eq!(
        entries.map(|entry| entry.message("value", "add")),
        [
            "resolver value symbol 'add' has parameter count metadata, expected none",
            "resolver value symbol 'add' has parameter names metadata, expected none",
            "resolver value symbol 'add' has parameter types metadata, expected none",
            "resolver value symbol 'add' has typed parameter types metadata, expected none",
            "resolver value symbol 'add' has return type metadata, expected none",
            "resolver value symbol 'add' has typed return type metadata, expected none",
        ]
    );
}

#[test]
fn value_signature_absence_validation_uses_module_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E7265");
    assert_eq!(validation.parameter_name_code, "E7267");
    assert_eq!(validation.parameter_type_name_code, "E7268");
    assert_eq!(validation.parameter_type_code, "E7371");
    assert_eq!(validation.return_type_code, "E7266");
    assert_eq!(validation.typed_return_type_code, "E7372");
}

#[test]
fn value_signature_absence_validation_uses_import_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E7281");
    assert_eq!(validation.parameter_name_code, "E7283");
    assert_eq!(validation.parameter_type_name_code, "E7284");
    assert_eq!(validation.parameter_type_code, "E7362");
    assert_eq!(validation.return_type_code, "E7282");
    assert_eq!(validation.typed_return_type_code, "E7363");
}

#[test]
fn value_signature_absence_validation_uses_local_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E7249");
    assert_eq!(validation.parameter_name_code, "E7251");
    assert_eq!(validation.parameter_type_name_code, "E7252");
    assert_eq!(validation.parameter_type_code, "E7380");
    assert_eq!(validation.return_type_code, "E7250");
    assert_eq!(validation.typed_return_type_code, "E7381");
}

#[test]
fn value_signature_absence_validation_uses_type_like_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E7310");
    assert_eq!(validation.parameter_name_code, "E7312");
    assert_eq!(validation.parameter_type_name_code, "E7313");
    assert_eq!(validation.parameter_type_code, "E7360");
    assert_eq!(validation.return_type_code, "E7311");
    assert_eq!(validation.typed_return_type_code, "E7361");
}

#[test]
fn value_signature_absence_validation_uses_variant_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E7330");
    assert_eq!(validation.parameter_name_code, "E7332");
    assert_eq!(validation.parameter_type_name_code, "E7333");
    assert_eq!(validation.parameter_type_code, "E7389");
    assert_eq!(validation.return_type_code, "E7331");
    assert_eq!(validation.typed_return_type_code, "E7390");
}
