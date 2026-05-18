use super::*;

#[path = "inference_conflicts/methods.rs"]
mod methods;

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

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose`: inferred `i32` and `str`"
        )),
        "expected generic function inference conflict diagnostic, got {errors:?}"
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

#[test]
fn generic_function_inference_conflict_through_function_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_with<T> = (left: T, mapper: (T) T) T {
    left
}

main = () i32 {
    mapper = (value: str) str {
        value
    }
    choose_with(1, mapper)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_with`: inferred `i32` and `str`"
        )),
        "expected generic function function-type inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_conflict_through_array_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_array<T> = (left: T, items: [T; 1]) T {
    left
}

main = () i32 {
    items = ["bad"]
    choose_array(1, items)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_array`: inferred `i32` and `str`"
        )),
        "expected generic function array-type inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_conflict_through_raw_pointer_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_raw<T> = (left: T, ptr: RawPtr<T>) T {
    left
}

main = () i32 {
    ptr = cast("bad", RawPtr<str>)
    choose_raw(1, ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_raw`: inferred `i32` and `str`"
        )),
        "expected generic function raw-pointer inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_conflict_through_slice_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_slice<T> = (left: T, items: Slice<T>) T {
    left
}

main = () i32 {
    items = cast("bad", Slice<str>)
    choose_slice(1, items)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_slice`: inferred `i32` and `str`"
        )),
        "expected generic function slice inference conflict diagnostic, got {errors:?}"
    );
}
