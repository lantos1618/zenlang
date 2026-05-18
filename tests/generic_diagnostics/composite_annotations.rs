use super::*;

#[test]
fn nested_generic_annotation_inner_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

read = (box: Box<Option<i32, StaticString>>) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected nested generic annotation inner arity diagnostic, got {errors:?}"
    );
}

#[test]
fn nested_generic_instantiation_inner_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

main = () i32 {
    value = Box<Option<i32, StaticString>> { value: Option<i32>.Some(1) }
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected nested generic instantiation inner arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_type_parameter_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

call = (f: (Box<i32, StaticString>) i32) i32 {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected function type parameter generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_type_return_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

factory = () () Box {
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected function type return generic annotation arity diagnostic, got {errors:?}"
    );
}

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
