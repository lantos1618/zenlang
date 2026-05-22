use super::*;

#[test]
fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

Point.requires(Json<StaticString>)
Point.requires(Json<i32>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("distinct behavior specializations should not overlap");
}
