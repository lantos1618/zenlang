use super::*;

#[test]
fn generic_function_type_arg_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T {
    value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = identity<Box<i32, StaticString>>(box)
    bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic function type-argument annotation arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 1")),
        "malformed generic function type argument should not also report argument mismatch, got {errors:?}"
    );
}

#[test]
fn generic_method_type_arg_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Holder: {
    value: i32
}

Holder.wrap<T> = (self: Holder, value: T) T {
    value
}

main = () i32 {
    holder = Holder { value: 1 }
    box = Box<i32> { value: 1 }
    bad = holder.wrap<Box<i32, StaticString>>(box)
    bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic method type-argument annotation arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "malformed generic method type argument should not also report argument mismatch, got {errors:?}"
    );
}

#[test]
fn generic_method_type_arg_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Holder: {
    value: i32
}

Holder.wrap<T> = (self: Holder, value: T) T {
    value
}

main = () i32 {
    holder = Holder { value: 1 }
    box = Box<i32> { value: 1 }
    bad = holder.wrap<Box>(box)
    bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected generic method type-argument annotation without args diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "malformed generic method type argument should not also report argument mismatch, got {errors:?}"
    );
}

#[test]
fn closure_param_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = (box: Box<i32, StaticString>) i32 {
        0
    }
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected closure parameter generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn closure_return_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = () Box {
        Box<i32> { value: 1 }
    }
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected closure return generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn cast_target_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box<i32, StaticString>)
    bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected cast target generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn cast_target_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected cast target generic annotation arity diagnostic, got {errors:?}"
    );
}
