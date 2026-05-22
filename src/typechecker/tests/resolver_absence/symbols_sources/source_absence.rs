use super::*;

#[test]
fn source_absence_validation_builds_source_validation() {
    let validation = SourceAbsenceValidation { code: "SOURCE" }.source_validation();

    assert_eq!(validation.code, "SOURCE");
    assert_eq!(validation.actual_missing, "none");
    assert_eq!(validation.expected_missing, "none");
    assert!(!validation.quote_expected);
}

#[test]
fn source_absence_validation_uses_type_like_resolver_code() {
    let validation = SourceAbsenceValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0309");
}

#[test]
fn source_absence_validation_uses_variant_resolver_code() {
    let validation = SourceAbsenceValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0329");
}

#[test]
fn source_absence_validation_uses_value_resolver_code() {
    let validation = SourceAbsenceValidation::value_resolver_code();

    assert_eq!(validation.code, "E0297");
}
