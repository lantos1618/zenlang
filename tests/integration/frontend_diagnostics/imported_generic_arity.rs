use super::support::{compile_to_c_panic_message, write_tmp_module};

#[test]
fn imported_generic_enum_method_explicit_type_arg_arity_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "result.zen",
        r#"
pub Result<T, E>:
    Ok(T),
    Err(E)

pub Result.unwrap_or<T, E> = (self: Self, fallback: T) T {
    self ?
        | Ok(value) { value }
        | Err(_) { fallback }
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ Result } = result

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.unwrap_or<i32>(0)
}
"#,
    );
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("generic method `Result.unwrap_or` expects 2 type arguments, found 1"),
        "expected imported generic method arity diagnostic, panic={message}"
    );
    assert!(
        !message.contains("cannot infer type argument"),
        "imported generic method arity failure should not also report inference, panic={message}"
    );
    assert!(
        !message.contains("argument 2"),
        "imported generic method arity failure should not also report argument mismatch, panic={message}"
    );
}

#[test]
fn imported_generic_function_explicit_type_arg_arity_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "helpers.zen",
        r#"
pub take_second<T, U> = (value: U) U {
    value
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ take_second } = helpers

main = () i32 {
    take_second<i32>(1)
}
"#,
    );
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("generic function `take_second` expects 2 type arguments, found 1"),
        "expected imported generic function arity diagnostic, panic={message}"
    );
    assert!(
        !message.contains("cannot infer type argument"),
        "imported generic function arity failure should not also report inference, panic={message}"
    );
    assert!(
        !message.contains("argument 1"),
        "imported generic function arity failure should not also report argument mismatch, panic={message}"
    );
}

#[test]
fn imported_generic_aggregate_constructor_type_arg_arity_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "types.zen",
        r#"
pub Box<T>: {
    value: T
}

pub Option<T>:
    Some(T),
    None
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ Box, Option } = types

main = () i32 {
    boxed = Box<i32, StaticString> { value: 1 }
    value = Option<i32, StaticString>.Some(1)
    boxed.value
}
"#,
    );
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("generic struct `Box` expects 1 type arguments, found 2"),
        "expected imported generic struct constructor arity diagnostic, panic={message}"
    );
    assert!(
        message.contains("generic enum `Option` expects 1 type arguments, found 2"),
        "expected imported generic enum constructor arity diagnostic, panic={message}"
    );
    assert!(
        !message.contains("field `value` for struct `Box`"),
        "imported generic struct constructor arity failure should not also report field mismatch, panic={message}"
    );
    assert!(
        !message.contains("payload for enum variant"),
        "imported generic enum constructor arity failure should not also report payload mismatch, panic={message}"
    );
}
