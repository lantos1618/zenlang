use super::*;

#[test]
fn generic_struct_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
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

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic struct bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
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

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic enum bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
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

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic struct annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
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

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic enum annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_local_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
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
    encode: (Self) str
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

#[test]
fn generic_function_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

encode<T: Json> = (value: T) str {
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

#[test]
fn generic_method_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Holder: {
    value: i32
}

Holder.wrap<T: Json> = (self: Holder, value: T) T {
    value
}

main = () i32 {
    holder = Holder { value: 1 }
    point = Point { x: 1 }
    bad = holder.wrap(point)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic method bound diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("has no method `encode`")),
        "generic method bound failure should not also specialize body method errors, got {errors:?}"
    );
}

#[test]
fn generic_receiver_method_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Box<T>: {
    value: T
}

Box.map<U: Json> = (self: Box<i32>, value: U) U {
    value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    point = Point { x: 1 }
    bad = box.map(point)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `U`")),
        "expected generic receiver method bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_ufc_function_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

as_json<T: Json> = (value: T) str {
    value.encode()
}

main = () i32 {
    point = Point { x: 1 }
    text = point.as_json()
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic UFC function bound diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("has no method `encode`")),
        "generic UFC bound failure should not also specialize body method errors, got {errors:?}"
    );
}
