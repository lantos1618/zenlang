use super::super::super::*;

#[test]
fn generic_function_inference_conflict_through_function_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_with<T> = (left: T, mapper: (T) T) T {
    left
}

main = () i32 {
    mapper = (value: StaticString) StaticString {
        value
    }
    choose_with(1, mapper)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_with`: inferred `i32` and `StaticString`"
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
            "conflicting inferred type argument `T` for generic function `choose_array`: inferred `i32` and `StaticString`"
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
    ptr = cast("bad", RawPtr<StaticString>)
    choose_raw(1, ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_raw`: inferred `i32` and `StaticString`"
        )),
        "expected generic function raw-pointer inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_conflict_through_pointer_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_ptr<T> = (left: T, ptr: Ptr<T>) T {
    left
}

main = () i32 {
    ptr = cast("bad", Ptr<StaticString>)
    choose_ptr(1, ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_ptr`: inferred `i32` and `StaticString`"
        )),
        "expected generic function pointer inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_conflict_through_mut_pointer_type_is_error() {
    let errors = typecheck_errors(
        r#"
choose_mut_ptr<T> = (left: T, ptr: MutPtr<T>) T {
    left
}

main = () i32 {
    ptr = cast("bad", MutPtr<StaticString>)
    choose_mut_ptr(1, ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_mut_ptr`: inferred `i32` and `StaticString`"
        )),
        "expected generic function mutable pointer inference conflict diagnostic, got {errors:?}"
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
    items = cast("bad", Slice<StaticString>)
    choose_slice(1, items)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_slice`: inferred `i32` and `StaticString`"
        )),
        "expected generic function slice inference conflict diagnostic, got {errors:?}"
    );
}
