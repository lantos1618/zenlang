use super::super::*;

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

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose`: inferred `i32` and `str`"
        )),
        "expected generic method inference conflict diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "generic method inference conflict should not also report argument mismatch, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("return type mismatch")),
        "generic method inference conflict should not also report return mismatch, got {errors:?}"
    );
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

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.replace`: inferred `i32` and `str`"
        )),
        "expected generic method receiver inference conflict diagnostic, got {errors:?}"
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
    value = Result<i32, str>.Ok(1)
    value.unwrap_or("bad")
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Result.unwrap_or`: inferred `i32` and `str`"
        )),
        "expected generic Result enum method receiver inference conflict diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 1")),
        "generic Result enum method inference conflict should not also report argument mismatch, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("return type mismatch")),
        "generic Result enum method inference conflict should not also report return mismatch, got {errors:?}"
    );
}
