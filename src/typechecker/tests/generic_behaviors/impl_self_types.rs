use super::*;

#[test]
fn behavior_impl_required_method_accepts_self_parameter() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Self) StaticString { "point" }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("behavior impl method may use Self for the impl target");
}

#[test]
fn behavior_impl_required_method_accepts_nested_self_types() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Holder<T>: { value: T }

Json: behavior {
    by_ptr: (Ptr<Self>) StaticString
    by_array: ([Self; 1]) StaticString
    by_function: ((Self) Self) StaticString
    wrap: (Self) Holder<Self>
}

Point.implements(Json) {
    by_ptr = (value: Ptr<Point>) StaticString { "ptr" }
    by_array = (value: [Point; 1]) StaticString { "array" }
    by_function = (value: (Point) Point) StaticString { "function" }
    wrap = (value: Point) Holder<Point> { Holder<Point> { value: value } }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("behavior impl signatures may use concrete target types inside Self-shaped types");
}
