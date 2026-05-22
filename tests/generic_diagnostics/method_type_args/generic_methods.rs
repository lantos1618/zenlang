use super::*;

#[test]
fn generic_method_explicit_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    box.get<i32, StaticString>()
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Box.get` expects 1 type arguments, found 2")),
        "expected generic method arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_inference_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.make<T> = (self: Box) T {
    self.value
}

main = () i32 {
    box = Box { value: 1 }
    box.make()
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot infer type argument `T` for generic method `Box.make`")),
        "expected generic method inference diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_argument_arity_uses_method_diagnostic() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    box.get(2)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `Box.get` expects 1 arguments, found 2")),
        "expected generic method arity diagnostic to name method kind, got {errors:?}"
    );
}
