use super::*;

#[test]
fn behavior_impl_for_unknown_type_is_error() {
    let errors = frontend_errors(
        r#"
Json: behavior {
    to_json: (Self) StaticString
}

Missing.implements(Json) {
    to_json = (value: Missing) StaticString {
        "missing"
    }
}
"#,
    );

    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown behavior impl target diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_for_unspecialized_generic_type_is_error() {
    let errors = frontend_errors(
        r#"
Box<T>: {
    value: T
}

Json: behavior {
    to_json: (Self) StaticString
}

Box.implements(Json) {
    to_json = (value: Box) StaticString {
        "box"
    }
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic type `Box` expects 1 type arguments, found 0")),
        "expected generic impl target arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_unspecialized_generic_type_is_error() {
    let errors = frontend_errors(
        r#"
Box<T>: {
    value: T
}

Json: behavior {
    to_json: (Self) StaticString
}

Box.requires(Json)
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic type `Box` expects 1 type arguments, found 0")),
        "expected generic requires target arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_extra_method_is_error() {
    let errors = typecheck_errors(
        r#"
Point: {
    x: i32
}

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString {
        "point"
    }

    extra = (value: Point) StaticString {
        "extra"
    }
}
"#,
    );

    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("method `extra` is not declared by behavior `Json`")
        }),
        "expected extra behavior impl method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_duplicate_method_is_error() {
    let errors = frontend_errors(
        r#"
Point: {
    x: i32
}

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString {
        "point"
    }

    to_json = (value: Point) StaticString {
        "point again"
    }
}
"#,
    );

    assert!(
        errors
            .iter()
            .any(|d| { d.message.contains("duplicate value symbol 'Point.to_json'") }),
        "expected duplicate behavior impl method diagnostic, got {errors:?}"
    );
}
