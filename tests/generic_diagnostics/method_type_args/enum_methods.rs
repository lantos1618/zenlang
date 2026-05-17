use super::*;

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
