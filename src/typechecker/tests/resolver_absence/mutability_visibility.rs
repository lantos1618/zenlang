use super::*;

#[test]
fn mutability_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
main = (mut input: i32) i32 {
    value ::= input
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup_scoped(Namespace::Local, "input")
        .expect("local symbol");
    let entries = MutabilityAbsenceValidation { code: "MUTABLE" }.entries(symbol);

    assert_eq!(
        entries,
        [AbsentMetadataEntry::new(true, "MUTABLE", "mutability")]
    );
}

#[test]
fn mutability_absence_validation_uses_module_resolver_code() {
    let validation = MutabilityAbsenceValidation::module_resolver_code();

    assert_eq!(validation.code, "E0345");
}

#[test]
fn mutability_absence_validation_uses_import_resolver_code() {
    let validation = MutabilityAbsenceValidation::import_resolver_code();

    assert_eq!(validation.code, "E0344");
}

#[test]
fn mutability_absence_validation_uses_type_like_resolver_code() {
    let validation = MutabilityAbsenceValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0314");
}

#[test]
fn mutability_absence_validation_uses_variant_resolver_code() {
    let validation = MutabilityAbsenceValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0343");
}

#[test]
fn mutability_absence_validation_uses_value_resolver_code() {
    let validation = MutabilityAbsenceValidation::value_resolver_code();

    assert_eq!(validation.code, "E0308");
}

#[test]
fn mutability_validation_formats_actual_and_expected() {
    let validation = MutabilityValidation { code: "MUTABLE" };

    assert_eq!(validation.code, "MUTABLE");
    assert_eq!(
        validation.display(Some(false), true),
        ("immutable", "mutable")
    );
    assert_eq!(validation.display(None, false), ("unknown", "immutable"));
    assert_eq!(
        validation.message("local", "value", Some(false), true),
        "resolver local symbol 'value' has mutability immutable, expected mutable"
    );
}

#[test]
fn mutability_validation_uses_resolver_code() {
    let validation = MutabilityValidation::resolver_code();

    assert_eq!(validation.code, "E0231");
}

#[test]
fn visibility_validation_formats_actual_and_expected() {
    let validation = VisibilityValidation { code: "VISIBLE" };

    assert_eq!(validation.code, "VISIBLE");
    assert_eq!(validation.display(true, false), ("public", "private"));
    assert_eq!(validation.display(false, true), ("private", "public"));
    assert_eq!(
        validation.message("import", "io", true, false),
        "resolver import symbol 'io' has visibility public, expected private"
    );
}

#[test]
fn visibility_validation_uses_local_resolver_code() {
    let validation = VisibilityValidation::local_resolver_code();

    assert_eq!(validation.code, "E0247");
}

#[test]
fn visibility_validation_uses_module_resolver_code() {
    let validation = VisibilityValidation::module_resolver_code();

    assert_eq!(validation.code, "E0229");
}

#[test]
fn visibility_validation_uses_import_resolver_code() {
    let validation = VisibilityValidation::import_resolver_code();

    assert_eq!(validation.code, "E0245");
}

#[test]
fn visibility_validation_uses_type_like_resolver_code() {
    let validation = VisibilityValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0225");
}

#[test]
fn visibility_validation_uses_variant_resolver_code() {
    let validation = VisibilityValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0226");
}

#[test]
fn visibility_validation_uses_value_resolver_code() {
    let validation = VisibilityValidation::value_resolver_code();

    assert_eq!(validation.code, "E0224");
}
