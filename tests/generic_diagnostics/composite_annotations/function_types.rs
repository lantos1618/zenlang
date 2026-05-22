use super::*;

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
