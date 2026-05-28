use super::*;

const JSON_POINT_PREAMBLE: &str = r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}
"#;

#[test]
fn generic_point_json_bound_failures_are_errors() {
    for (declaration, use_site, context) in [
        (
            r#"
Box<T: Json>: {
    value: T
}
"#,
            r#"
main = () i32 {
    point = Point { x: 1 }
    box = Box<Point> { value: point }
    box.value.x
}
"#,
            "generic struct bound",
        ),
        (
            r#"
Option<T: Json>:
    None,
    Some(T)
"#,
            r#"
main = () i32 {
    point = Point { x: 1 }
    value = Option<Point>.Some(point)
    0
}
"#,
            "generic enum bound",
        ),
        (
            r#"
Box<T: Json>: {
    value: T
}
"#,
            r#"
read = (box: Box<Point>) i32 {
    box.value.x
}
"#,
            "generic struct annotation bound",
        ),
        (
            r#"
Option<T: Json>:
    None,
    Some(T)
"#,
            r#"
read = (value: Option<Point>) i32 {
    0
}
"#,
            "generic enum annotation bound",
        ),
        (
            r#"
Box<T: Json>: {
    value: T
}
"#,
            r#"
main = () i32 {
    point = Point { x: 1 }
    box: Box<Point> = Box<Point> { value: point }
    box.value.x
}
"#,
            "generic struct local annotation bound",
        ),
        (
            r#"
Option<T: Json>:
    None,
    Some(T)
"#,
            r#"
main = () i32 {
    point = Point { x: 1 }
    value: Option<Point> = Option<Point>.Some(point)
    0
}
"#,
            "generic enum local annotation bound",
        ),
    ] {
        let errors = typecheck_errors(&format!("{JSON_POINT_PREAMBLE}\n{declaration}\n{use_site}"));
        assert_point_json_bound_failure(&errors, "T", context);
    }
}
