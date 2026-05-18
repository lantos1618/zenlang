use super::super::super::*;

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
fn generic_method_inference_conflict_through_pointer_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose_ptr<T> = (self: Box<T>, ptr: Ptr<T>) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    ptr = cast("bad", Ptr<str>)
    box.choose_ptr(ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose_ptr`: inferred `i32` and `str`"
        )),
        "expected generic method pointer inference conflict diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_inference_conflict_through_mut_pointer_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.choose_mut_ptr<T> = (self: Box<T>, ptr: MutPtr<T>) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    ptr = cast("bad", MutPtr<str>)
    box.choose_mut_ptr(ptr)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic method `Box.choose_mut_ptr`: inferred `i32` and `str`"
        )),
        "expected generic method mutable pointer inference conflict diagnostic, got {errors:?}"
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
