use super::*;

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
