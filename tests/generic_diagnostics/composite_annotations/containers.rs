use super::*;

#[test]
fn pointer_type_inner_generic_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (ptr: Ptr<Box<i32, StaticString>>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected pointer inner generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn slice_type_inner_generic_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (slice: Slice<Box>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected slice inner generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn array_type_inner_generic_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (items: [Box<i32, StaticString>; 1]) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected array inner generic annotation arity diagnostic, got {errors:?}"
    );
}
