use super::*;

#[test]
fn field_validation_formats_messages() {
    let validation = FieldValidation {
        display_code: "FIELDS".into(),
        typed_code: "TYPED_FIELDS".into(),
    };

    assert_eq!(validation.display_code, "FIELDS");
    assert_eq!(
        validation.display_message("type", "Point", "(x: i32)", "(x: f64)"),
        "resolver type symbol 'Point' has fields '(x: i32)', expected '(x: f64)'"
    );
    assert_eq!(
        validation.typed_message(
            "type",
            "Pipeline",
            "(callback: i32)",
            "(callback: (i32) i32)",
        ),
        "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
    );
}

#[test]
fn field_validation_uses_resolver_codes() {
    let validation = FieldValidation::resolver_codes();

    assert_eq!(validation.display_code, "E7217");
    assert_eq!(validation.typed_code, "E7358");
}

#[test]
fn variant_payload_validation_formats_messages() {
    let validation = VariantPayloadValidation {
        display_code: "PAYLOAD".into(),
        typed_code: "TYPED_PAYLOAD".into(),
    };

    assert_eq!(validation.display_code, "PAYLOAD");
    assert_eq!(
        validation.display_message("Some", "bool", "i32"),
        "resolver variant symbol 'Some' has payload type 'bool', expected 'i32'"
    );
    assert_eq!(
        validation.typed_message("Wrap", "i32", "(i32) i32"),
        "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
    );
}

#[test]
fn variant_payload_validation_uses_resolver_codes() {
    let validation = VariantPayloadValidation::resolver_codes();

    assert_eq!(validation.display_code, "E7218");
    assert_eq!(validation.typed_code, "E7359");
}

#[test]
fn variant_owner_validation_formats_message() {
    let validation = VariantOwnerValidation {
        code: "OWNER".into(),
    };

    assert_eq!(validation.code, "OWNER");
    assert_eq!(
        validation.message("Some", "Result", "Option"),
        "resolver variant symbol 'Some' has owner 'Result', expected 'Option'"
    );
}

#[test]
fn variant_owner_validation_uses_resolver_code() {
    let validation = VariantOwnerValidation::resolver_code();

    assert_eq!(validation.code, "E7242");
}

#[test]
fn variant_name_validation_formats_message() {
    let validation = VariantNameValidation {
        code: "VARIANTS".into(),
    };

    assert_eq!(validation.code, "VARIANTS");
    assert_eq!(
        validation.message("Option", "(Some)", "(Some, None)"),
        "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'"
    );
}

#[test]
fn variant_name_validation_uses_resolver_code() {
    let validation = VariantNameValidation::resolver_code();

    assert_eq!(validation.code, "E7241");
}
