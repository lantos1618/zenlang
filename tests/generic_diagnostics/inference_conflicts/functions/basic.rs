use super::super::super::*;

#[test]
fn generic_function_inference_conflict_is_error() {
    let errors = typecheck_errors(
        r#"
choose<T> = (left: T, right: T) T {
    left
}

main = () i32 {
    value = choose(1, "bad")
    value
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "function",
        "choose",
        "T",
        "i32",
        "StaticString",
        "generic function inference conflict",
    );
    assert_no_diagnostic_message(&errors, "argument 2", "function inference conflict");
    assert_no_diagnostic_message(
        &errors,
        "return type mismatch",
        "function inference conflict",
    );
}
