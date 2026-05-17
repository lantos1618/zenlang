use super::*;

#[path = "method_type_args/arity_followups.rs"]
mod arity_followups;

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
    box.get<i32, str>()
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
fn generic_enum_method_explicit_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

Option.unwrap_or<T> = (self: Self, fallback: T) T {
    self ?
        | Some(value) { value }
        | None { fallback }
}

main = () i32 {
    value = Option<i32>.Some(1)
    value.unwrap_or<i32, str>(0)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Option.unwrap_or` expects 1 type arguments, found 2")),
        "expected generic enum method arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "malformed generic enum method type arguments should not also report argument mismatch, got {errors:?}"
    );
}

#[test]
fn generic_result_enum_method_explicit_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Result<T, E>:
    Ok(T),
    Err(E)

Result.unwrap_or<T, E> = (self: Self, fallback: T) T {
    self ?
        | Ok(value) { value }
        | Err(_) { fallback }
}

main = () i32 {
    value = Result<i32, str>.Ok(1)
    value.unwrap_or<i32>(0)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Result.unwrap_or` expects 2 type arguments, found 1")),
        "expected generic Result enum method arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("cannot infer type argument")),
        "malformed generic Result enum method type arguments should not also report inference, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "malformed generic Result enum method type arguments should not also report argument mismatch, got {errors:?}"
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
