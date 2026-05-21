use super::*;

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
