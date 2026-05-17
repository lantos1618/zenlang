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
    bad = identity<Box<i32, str>>(box)
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
    bad = holder.wrap<Box<i32, str>>(box)
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
}

#[test]
fn closure_param_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = (box: Box<i32, str>) i32 {
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
    bad = cast(box, Box<i32, str>)
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
fn as_cast_target_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = box as Box
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected as-cast target generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (box: Box<i32, str>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic struct annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option<i32, str>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected generic enum annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn nongeneric_struct_annotation_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
Point: {
    x: i32
}

read = (point: Point<i32>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic struct `Point` does not accept type arguments")),
        "expected non-generic struct annotation type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("generic struct `Point` expects 0")),
        "non-generic struct annotation should not use generic arity wording, got {errors:?}"
    );
}

#[test]
fn nongeneric_enum_annotation_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
Status:
    Ready,
    Done(i32)

read = (value: Status<i32>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic enum `Status` does not accept type arguments")),
        "expected non-generic enum annotation type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("generic enum `Status` expects 0")),
        "non-generic enum annotation should not use generic arity wording, got {errors:?}"
    );
}

#[test]
fn generic_struct_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (box: Box) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected unspecialized generic struct annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 0")),
        "expected unspecialized generic enum annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_local_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box<i32, str> = Box<i32> { value: 1 }
    box.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic struct local annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_local_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box = Box<i32> { value: 1 }
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected unspecialized generic struct local annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_local_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value: Option<i32, str> = Option<i32>.Some(1)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected generic enum local annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_local_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value: Option = Option<i32>.Some(1)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 0")),
        "expected unspecialized generic enum local annotation diagnostic, got {errors:?}"
    );
}
