use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_result_method_arity_schema_matches_golden() {
    assert_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_result_method_arity.golden.json",
        "generic method arity",
        1,
        "generic arity diagnostics should not emit inference or argument followups",
    );
}

#[test]
fn emit_json_diagnostics_generic_result_method_bound_schema_matches_golden() {
    assert_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_result_method_bound.golden.json",
        "generic method bound",
        1,
        "generic bound diagnostics should not emit method-body followups",
    );
}

#[test]
fn emit_json_diagnostics_generic_function_bound_schema_matches_golden() {
    assert_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_function_bound.golden.json",
        "generic function bound",
        1,
        "generic function bound diagnostics should not emit method-body followups",
    );
}

#[test]
fn emit_json_diagnostics_generic_result_method_inference_schema_matches_golden() {
    assert_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_result_method_inference.golden.json",
        "generic method inference",
        1,
        "generic inference diagnostics should not emit argument or return followups",
    );
}

#[test]
fn emit_json_diagnostics_generic_function_inference_schema_matches_golden() {
    assert_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_function_inference.golden.json",
        "generic function inference",
        1,
        "generic function inference diagnostics should not emit argument or return followups",
    );
}
