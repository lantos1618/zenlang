use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_method_schemas_match_golden() {
    for (filename, source, description, count_context) in [
        (
            "generic_result_method_arity.zen",
            r#"
Result<T, E>:
    Ok(T),
    Err(E)

Result.unwrap_or<T, E> = (self: Self, fallback: T) T {
    self ?
        | Ok(value) { value }
        | Err(_) { fallback }
}

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.unwrap_or<i32>(0)
}
"#,
            "generic method arity",
            "generic arity diagnostics should not emit inference or argument followups",
        ),
        (
            "generic_result_method_bound.zen",
            r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}

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
            "generic method bound",
            "generic bound diagnostics should not emit method-body followups",
        ),
        (
            "generic_function_bound.zen",
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
            "generic function bound",
            "generic function bound diagnostics should not emit method-body followups",
        ),
        (
            "generic_enum_constructor_bound.zen",
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
            "generic enum constructor bound",
            "generic enum constructor bound diagnostics should not emit constructor followups",
        ),
        (
            "generic_result_method_inference.zen",
            r#"
Result<T, E>:
    Ok(T),
    Err(E)

Result.unwrap_or<T, E> = (self: Self, fallback: T) T {
    self ?
        | Ok(value) { value }
        | Err(_) { fallback }
}

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.unwrap_or("bad")
}
"#,
            "generic method inference",
            "generic inference diagnostics should not emit argument or return followups",
        ),
        (
            "generic_function_inference.zen",
            r#"
choose<T> = (left: T, right: T) T {
    left
}

main = () i32 {
    value = choose(1, "bad")
    value
}
"#,
            "generic function inference",
            "generic function inference diagnostics should not emit argument or return followups",
        ),
        (
            "generic_function_inference_failure.zen",
            r#"
make_default<T> = () T {
    0
}

main = () i32 {
    make_default()
}
"#,
            "generic function inference failure",
            "generic function inference failure diagnostics should not emit return followups",
        ),
        (
            "generic_method_inference_failure.zen",
            r#"
Box: {
    value: i32
}

Box.make<T> = (self: Box) T {
    self.value
}

main = () i32 {
    box = Box { value: 1 }
    box.make()
}
"#,
            "generic method inference failure",
            "generic method inference failure diagnostics should not emit return followups",
        ),
    ] {
        assert_diagnostics_golden(filename, source, description, 1, count_context);
    }
}
