use super::*;

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
