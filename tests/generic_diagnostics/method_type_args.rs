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

#[test]
fn generic_function_explicit_type_arg_arity_does_not_emit_inference_followup() {
    let errors = typecheck_errors(
        r#"
pick<T, U> = (value: T) T {
    value
}

main = () i32 {
    pick<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `pick` expects 2 type arguments, found 1")),
        "expected explicit generic arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("cannot infer type argument")),
        "explicit generic arity failure should not also report inference, got {errors:?}"
    );
}

#[test]
fn generic_method_explicit_type_arg_arity_does_not_emit_inference_followup() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.pick<T, U> = (self: Box, value: T) T {
    value
}

main = () i32 {
    box = Box { value: 1 }
    box.pick<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Box.pick` expects 2 type arguments, found 1")),
        "expected explicit generic method arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("cannot infer type argument")),
        "explicit generic method arity failure should not also report inference, got {errors:?}"
    );
}

#[test]
fn generic_function_explicit_type_arg_arity_does_not_emit_argument_followup() {
    let errors = typecheck_errors(
        r#"
take_second<T, U> = (value: U) U {
    value
}

main = () i32 {
    take_second<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `take_second` expects 2 type arguments, found 1")),
        "expected explicit generic arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 1")),
        "explicit generic arity failure should not also report argument mismatch, got {errors:?}"
    );
}

#[test]
fn generic_method_explicit_type_arg_arity_does_not_emit_argument_followup() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.take_second<T, U> = (self: Box, value: U) U {
    value
}

main = () i32 {
    box = Box { value: 1 }
    box.take_second<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Box.take_second` expects 2 type arguments, found 1")),
        "expected explicit generic method arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "explicit generic method arity failure should not also report argument mismatch, got {errors:?}"
    );
}
