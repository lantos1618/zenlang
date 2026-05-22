use super::*;

#[test]
fn generic_struct_local_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}

Box<T: Json>: {
    value: T
}

main = () i32 {
    point = Point { x: 1 }
    box: Box<Point> = Box<Point> { value: point }
    box.value.x
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic struct local annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_local_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}

Option<T: Json>:
    None,
    Some(T)

main = () i32 {
    point = Point { x: 1 }
    value: Option<Point> = Option<Point>.Some(point)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic enum local annotation bound diagnostic, got {errors:?}"
    );
}
