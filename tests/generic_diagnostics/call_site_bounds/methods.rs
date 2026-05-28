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
fn generic_method_call_site_bound_failures_are_errors() {
    for (program, param, context, no_encode_context) in [
        (
            r#"
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
            "T",
            "generic method bound",
            Some("generic method bound"),
        ),
        (
            r#"
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
            "U",
            "generic receiver method bound",
            None,
        ),
        (
            r#"
Result<T, E>:
    Ok(T),
    Err(E)

Result.map<T, E, U: Json> = (self: Self, fallback: U) U {
    fallback.encode()
    fallback
}

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    point = Point { x: 1 }
    bad = value.map(point)
    0
}
"#,
            "U",
            "generic Result enum method bound",
            Some("generic Result enum method bound"),
        ),
        (
            r#"
as_json<T: Json> = (value: T) StaticString {
    value.encode()
}

main = () i32 {
    point = Point { x: 1 }
    text = point.as_json()
    0
}
"#,
            "T",
            "generic UFC function bound",
            Some("generic UFC bound"),
        ),
    ] {
        let errors = typecheck_errors(&format!("{JSON_POINT_PREAMBLE}\n{program}"));
        assert_point_json_bound_failure(&errors, param, context);
        if let Some(context) = no_encode_context {
            assert_no_diagnostic_message(&errors, "has no method `encode`", context);
        }
    }
}
