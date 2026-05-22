use super::*;

#[test]
fn nongeneric_method_explicit_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.get = (self: Box) i32 {
    self.value
}

main = () i32 {
    box = Box { value: 1 }
    box.get<i32>()
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic method `Box.get` does not accept type arguments")),
        "expected non-generic method type-argument diagnostic, got {errors:?}"
    );
}

#[test]
fn module_function_explicit_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
{ io } = std

main = () i32 {
    io.println<i32>("bad")
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic function `io.println` does not accept type arguments")),
        "expected module function type-argument diagnostic, got {errors:?}"
    );
}

#[test]
fn builtin_function_explicit_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
main = () i32 {
    @builtin.panic<i32>("bad")
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic function `@builtin.panic` does not accept type arguments")),
        "expected builtin function type-argument diagnostic, got {errors:?}"
    );
}
