use super::*;

#[path = "call_site_bounds/methods.rs"]
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

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic function bound diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("has no method `encode`")),
        "generic function bound failure should not also specialize body method errors, got {errors:?}"
    );
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

    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("type `Point` has no method `serialize`")),
        "expected unknown method diagnostic, got {errors:?}"
    );
}
