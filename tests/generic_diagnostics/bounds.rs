use super::*;

#[test]
fn generic_struct_behavior_bound_failure_is_error() {
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
    box = Box<Point> { value: point }
    box.value.x
}
"#,
    );

    assert_diagnostic_code_and_message(
        &errors,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "generic struct bound",
    );
}

#[test]
fn generic_enum_behavior_bound_failure_is_error() {
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
    value = Option<Point>.Some(point)
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &errors,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "generic enum bound",
    );
}

#[test]
fn generic_struct_annotation_bound_failure_is_error() {
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

read = (box: Box<Point>) i32 {
    box.value.x
}
"#,
    );

    assert_diagnostic_code_and_message(
        &errors,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "generic struct annotation bound",
    );
}

#[test]
fn generic_enum_annotation_bound_failure_is_error() {
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

read = (value: Option<Point>) i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &errors,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "generic enum annotation bound",
    );
}

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

    assert_diagnostic_code_and_message(
        &errors,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "generic struct local annotation bound",
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

    assert_diagnostic_code_and_message(
        &errors,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "generic enum local annotation bound",
    );
}
