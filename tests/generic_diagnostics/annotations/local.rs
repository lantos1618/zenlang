use super::*;

#[test]
fn generic_struct_local_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box<i32, StaticString> = Box<i32> { value: 1 }
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
    value: Option<i32, StaticString> = Option<i32>.Some(1)
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
