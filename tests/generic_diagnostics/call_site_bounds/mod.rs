use super::*;
mod methods;

#[test]
fn generic_function_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}

encode<T: Json> = (value: T) StaticString {
    value.encode()
}

main = () i32 {
    point = Point { x: 1 }
    text = encode(point)
    0
}
"#,
    );

    assert_point_json_bound_failure(&errors, "T", "generic function bound");
    assert_no_diagnostic_message(&errors, "has no method `encode`", "generic function bound");
}

#[test]
fn generic_behavior_bound_unknown_method_is_error() {
    let errors = typecheck_errors(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: {
    x: i32
}

Point.implements(Json<Point>) {
    encode = (value: Point) Point {
        value
    }
}

decode<T: Json<T>> = (value: T) T {
    value.serialize()
}

main = () i32 {
    point = Point { x: 1 }
    decoded = decode(point)
    decoded.x
}
"#,
    );

    assert_diagnostic_message(
        &errors,
        "type `Point` has no method `serialize`",
        "unknown method",
    );
}
