use super::super::super::*;

#[test]
fn generic_method_inference_conflict_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose<T> = (self: Box<T>, other: T) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    box.choose("bad")
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "method",
        "Box.choose",
        "T",
        "i32",
        "StaticString",
        "generic method inference conflict",
    );
    assert_no_diagnostic_message(&errors, "argument 2", "method inference conflict");
    assert_no_diagnostic_message(&errors, "return type mismatch", "method inference conflict");
}

#[test]
fn generic_method_inference_conflict_from_receiver_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.replace<T> = (self: Box<T>, value: T) T {
    value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    box.replace("bad")
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "method",
        "Box.replace",
        "T",
        "i32",
        "StaticString",
        "generic method receiver inference conflict",
    );
}

#[test]
fn generic_result_enum_method_inference_conflict_from_receiver_is_error() {
    let errors = typecheck_errors(
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
    );

    assert_inference_conflict(
        &errors,
        "method",
        "Result.unwrap_or",
        "T",
        "i32",
        "StaticString",
        "generic Result enum method receiver inference conflict",
    );
    assert_no_diagnostic_message(
        &errors,
        "argument 1",
        "Result enum method inference conflict",
    );
    assert_no_diagnostic_message(
        &errors,
        "return type mismatch",
        "Result enum method inference conflict",
    );
}
