use super::*;

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

#[test]
fn generic_method_inference_conflict_through_function_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose_with<T> = (self: Box<T>, mapper: (T) T) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    mapper = (value: str) str {
        value
    }
    box.choose_with(mapper)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose_with`: inferred `i32` and `str`"
        )),
        "expected generic method function-type inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_inference_conflict_through_array_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose_array<T> = (self: Box<T>, items: [T; 1]) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    items = ["bad"]
    box.choose_array(items)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose_array`: inferred `i32` and `str`"
        )),
        "expected generic method array-type inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_inference_conflict_through_raw_pointer_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose_raw<T> = (self: Box<T>, ptr: RawPtr<T>) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    ptr = cast("bad", RawPtr<str>)
    box.choose_raw(ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose_raw`: inferred `i32` and `str`"
        )),
        "expected generic method raw-pointer inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_inference_conflict_through_slice_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose_slice<T> = (self: Box<T>, items: Slice<T>) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    items = cast("bad", Slice<str>)
    box.choose_slice(items)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose_slice`: inferred `i32` and `str`"
        )),
        "expected generic method slice inference conflict diagnostic, got {errors:?}"
    );
}
