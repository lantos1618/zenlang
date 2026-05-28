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

    assert_diagnostic_message(
        &errors,
        "unknown type symbol 'Missing'",
        "behavior impl target",
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

    assert_diagnostic_message(
        &errors,
        "generic type `Box` expects 1 type arguments, found 0",
        "generic impl target arity",
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

    assert_diagnostic_message(
        &errors,
        "generic type `Box` expects 1 type arguments, found 0",
        "generic requires target arity",
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

    assert_diagnostic_message(
        &errors,
        "method `extra` is not declared by behavior `Json`",
        "extra behavior impl method",
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

    assert_diagnostic_message(
        &errors,
        "duplicate value symbol 'Point.to_json'",
        "duplicate behavior impl method",
    );
}
