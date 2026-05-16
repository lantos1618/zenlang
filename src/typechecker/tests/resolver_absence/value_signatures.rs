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
        parameter_count_code: "PARAM_COUNT",
        parameter_name_code: "PARAM_NAMES",
        parameter_type_name_code: "PARAM_TYPES",
        parameter_type_code: "TYPED_PARAM_TYPES",
        return_type_code: "RETURN_TYPE",
        typed_return_type_code: "TYPED_RETURN_TYPE",
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

    assert_eq!(validation.parameter_count_code, "E0265");
    assert_eq!(validation.parameter_name_code, "E0267");
    assert_eq!(validation.parameter_type_name_code, "E0268");
    assert_eq!(validation.parameter_type_code, "E0371");
    assert_eq!(validation.return_type_code, "E0266");
    assert_eq!(validation.typed_return_type_code, "E0372");
}

#[test]
fn value_signature_absence_validation_uses_import_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0281");
    assert_eq!(validation.parameter_name_code, "E0283");
    assert_eq!(validation.parameter_type_name_code, "E0284");
    assert_eq!(validation.parameter_type_code, "E0362");
    assert_eq!(validation.return_type_code, "E0282");
    assert_eq!(validation.typed_return_type_code, "E0363");
}

#[test]
fn value_signature_absence_validation_uses_local_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0249");
    assert_eq!(validation.parameter_name_code, "E0251");
    assert_eq!(validation.parameter_type_name_code, "E0252");
    assert_eq!(validation.parameter_type_code, "E0380");
    assert_eq!(validation.return_type_code, "E0250");
    assert_eq!(validation.typed_return_type_code, "E0381");
}

#[test]
fn value_signature_absence_validation_uses_type_like_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0310");
    assert_eq!(validation.parameter_name_code, "E0312");
    assert_eq!(validation.parameter_type_name_code, "E0313");
    assert_eq!(validation.parameter_type_code, "E0360");
    assert_eq!(validation.return_type_code, "E0311");
    assert_eq!(validation.typed_return_type_code, "E0361");
}

#[test]
fn value_signature_absence_validation_uses_variant_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0330");
    assert_eq!(validation.parameter_name_code, "E0332");
    assert_eq!(validation.parameter_type_name_code, "E0333");
    assert_eq!(validation.parameter_type_code, "E0389");
    assert_eq!(validation.return_type_code, "E0331");
    assert_eq!(validation.typed_return_type_code, "E0390");
}
