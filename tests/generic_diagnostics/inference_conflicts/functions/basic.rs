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

    assert_diagnostic_code_and_message(
        &errors,
        "E5000",
        "conflicting inferred type argument `T` for generic function `choose`: inferred `i32` and `StaticString`",
        "generic function inference conflict",
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "generic function inference conflict should not also report argument mismatch, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("return type mismatch")),
        "generic function inference conflict should not also report return mismatch, got {errors:?}"
    );
}
