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

#[test]
fn type_parameter_validation_formats_messages() {
    let validation = TypeParameterValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    };

    assert_eq!(validation.name_code, "NAMES");
    assert_eq!(
        validation.name_message("value", "identity", "(U)", "(T)"),
        "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
    );
    assert_eq!(
        validation.bound_message("type", "Box", "(T: Other)", "(T: Json)"),
        "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
    );
    assert_eq!(
            validation.bound_ref_message("behavior", "Serializable", "(T: Json<i32>)", "(T: Json<T>)"),
            "resolver behavior symbol 'Serializable' has type parameter bound refs '(T: Json<i32>)', expected '(T: Json<T>)'"
        );
}

#[test]
fn type_parameter_validation_uses_type_like_resolver_codes() {
    let validation = TypeParameterValidation::type_like_resolver_codes();

    assert_eq!(validation.count_code, "E0213");
    assert_eq!(validation.name_code, "E0346");
    assert_eq!(validation.bound_code, "E0222");
    assert_eq!(validation.bound_ref_code, "E0350");
}

#[test]
fn type_parameter_validation_uses_value_resolver_codes() {
    let validation = TypeParameterValidation::value_resolver_codes();

    assert_eq!(validation.count_code, "E0220");
    assert_eq!(validation.name_code, "E0347");
    assert_eq!(validation.bound_code, "E0221");
    assert_eq!(validation.bound_ref_code, "E0351");
}

#[test]
fn type_parameter_validation_builds_count_validation() {
    let validation = TypeParameterValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    }
    .count_validation();

    assert_eq!(validation.label, "type parameter count");
    assert_eq!(validation.code, "COUNT");
}

#[test]
fn value_parameter_validation_formats_messages() {
    let validation = ValueParameterValidation {
        name_code: "NAMES",
        display_type_code: "TYPES",
        typed_type_code: "TYPED_TYPES",
    };

    assert_eq!(validation.name_code, "NAMES");
    assert_eq!(
        validation.name_message("add", "(a, other)", "(a, b)"),
        "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
    );
    assert_eq!(
        validation.display_type_message("add", "(i32, i32)", "(i32, f64)"),
        "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
    );
    assert_eq!(
        validation.typed_type_message("apply", "(i32)", "((i32) i32)"),
        "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
    );
}

#[test]
fn value_parameter_validation_uses_resolver_codes() {
    let validation = ValueParameterValidation::resolver_codes();

    assert_eq!(validation.name_code, "E0223");
    assert_eq!(validation.display_type_code, "E0216");
    assert_eq!(validation.typed_type_code, "E0356");
}

#[test]
fn return_validation_formats_messages() {
    let validation = ReturnValidation {
        display_code: "RETURN",
        typed_code: "TYPED_RETURN",
    };

    assert_eq!(validation.display_code, "RETURN");
    assert_eq!(
        validation.display_message("main", "bool", "i32"),
        "resolver value symbol 'main' has return type 'bool', expected 'i32'"
    );
    assert_eq!(
        validation.typed_message("apply", "i32", "(i32) i32"),
        "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
    );
}

#[test]
fn return_validation_uses_resolver_codes() {
    let validation = ReturnValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0212");
    assert_eq!(validation.typed_code, "E0357");
}

#[test]
fn behavior_method_validation_formats_messages() {
    let validation = BehaviorMethodValidation {
        display_code: "METHODS",
        typed_code: "TYPED_METHODS",
    };

    assert_eq!(validation.display_code, "METHODS");
    assert_eq!(
            validation.display_message("Serializable", "(encode(Self) bool)", "(encode(Self) StaticString)"),
            "resolver behavior symbol 'Serializable' has methods '(encode(Self) bool)', expected '(encode(Self) StaticString)'"
        );
    assert_eq!(
            validation.typed_message(
                "Mapper",
                "(map(__arg0: Self, __arg1: i32) i32)",
                "(map(__arg0: Self, __arg1: (i32) i32) i32)"
            ),
            "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) i32)'"
        );
}

#[test]
fn behavior_method_validation_uses_resolver_codes() {
    let validation = BehaviorMethodValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0219");
    assert_eq!(validation.typed_code, "E0355");
}

#[test]
fn field_validation_formats_messages() {
    let validation = FieldValidation {
        display_code: "FIELDS",
        typed_code: "TYPED_FIELDS",
    };

    assert_eq!(validation.display_code, "FIELDS");
    assert_eq!(
        validation.display_message("type", "Point", "(x: i32)", "(x: f64)"),
        "resolver type symbol 'Point' has fields '(x: i32)', expected '(x: f64)'"
    );
    assert_eq!(
            validation.typed_message("type", "Pipeline", "(callback: i32)", "(callback: (i32) i32)"),
            "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
        );
}

#[test]
fn field_validation_uses_resolver_codes() {
    let validation = FieldValidation::resolver_codes();

    assert_eq!(validation.display_code, "E0217");
    assert_eq!(validation.typed_code, "E0358");
}

#[test]
fn variant_payload_validation_formats_messages() {
    let validation = VariantPayloadValidation {
        display_code: "PAYLOAD",
        typed_code: "TYPED_PAYLOAD",
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

    assert_eq!(validation.display_code, "E0218");
    assert_eq!(validation.typed_code, "E0359");
}

#[test]
fn variant_owner_validation_formats_message() {
    let validation = VariantOwnerValidation { code: "OWNER" };

    assert_eq!(validation.code, "OWNER");
    assert_eq!(
        validation.message("Some", "Result", "Option"),
        "resolver variant symbol 'Some' has owner 'Result', expected 'Option'"
    );
}

#[test]
fn variant_owner_validation_uses_resolver_code() {
    let validation = VariantOwnerValidation::resolver_code();

    assert_eq!(validation.code, "E0242");
}

#[test]
fn variant_name_validation_formats_message() {
    let validation = VariantNameValidation { code: "VARIANTS" };

    assert_eq!(validation.code, "VARIANTS");
    assert_eq!(
        validation.message("Option", "(Some)", "(Some, None)"),
        "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'"
    );
}

#[test]
fn variant_name_validation_uses_resolver_code() {
    let validation = VariantNameValidation::resolver_code();

    assert_eq!(validation.code, "E0241");
}
