use super::*;

#[path = "method_type_args/arity_followups.rs"]
mod arity_followups;
#[path = "method_type_args/enum_methods.rs"]
mod enum_methods;

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

    assert_nongeneric_type_args_diagnostic(
        &errors,
        "method",
        "Box.get",
        "non-generic method type-argument",
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

    assert_nongeneric_type_args_diagnostic(
        &errors,
        "function",
        "io.println",
        "module function type-argument",
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

    assert_nongeneric_type_args_diagnostic(
        &errors,
        "function",
        "@builtin.panic",
        "builtin function type-argument",
    );
}

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

    assert_generic_arity_diagnostic(&errors, "method", "Box.get", 1, 2, "generic method arity");
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
