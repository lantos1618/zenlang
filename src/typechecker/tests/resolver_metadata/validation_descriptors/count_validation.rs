use super::*;

#[test]
fn resolver_count_display_formats_known_and_missing_counts() {
    assert_eq!(resolver_count_display(Some(2)), "2");
    assert_eq!(resolver_count_display(None), "unknown");
}

#[test]
fn count_validation_formats_message() {
    let validation = CountValidation {
        label: "parameter count",
        code: "COUNT",
    };

    assert_eq!(validation.code, "COUNT");
    assert_eq!(
        validation.message("value", "add", Some(1), 2),
        "resolver value symbol 'add' has parameter count 1, expected 2"
    );
    assert_eq!(
        validation.message("variant", "Some", None, 1),
        "resolver variant symbol 'Some' has parameter count unknown, expected 1"
    );
}

#[test]
fn count_validation_uses_value_parameter_resolver_code() {
    let validation = CountValidation::value_parameter_resolver_code();

    assert_eq!(validation.label, "parameter count");
    assert_eq!(validation.code, "E0211");
}

#[test]
fn count_validation_uses_field_resolver_code() {
    let validation = CountValidation::field_resolver_code();

    assert_eq!(validation.label, "field count");
    assert_eq!(validation.code, "E0214");
}

#[test]
fn count_validation_uses_variant_payload_resolver_code() {
    let validation = CountValidation::variant_payload_resolver_code();

    assert_eq!(validation.label, "payload count");
    assert_eq!(validation.code, "E0215");
}
