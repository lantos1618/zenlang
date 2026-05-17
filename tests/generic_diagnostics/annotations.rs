use super::*;

#[path = "annotations/local.rs"]
mod local;

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
