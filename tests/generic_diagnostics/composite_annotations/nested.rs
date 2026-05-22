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
