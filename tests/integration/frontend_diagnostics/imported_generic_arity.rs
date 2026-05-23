use super::support::{
    assert_diagnostic_code_and_message, assert_no_diagnostic_message, frontend_diagnostics,
    write_tmp_module,
};

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
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic method `Result.unwrap_or` expects 2 type arguments, found 1",
        "imported generic method arity",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "cannot infer type argument",
        "imported generic method arity failure",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "argument 2",
        "imported generic method arity failure",
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
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic function `take_second` expects 2 type arguments, found 1",
        "imported generic function arity",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "cannot infer type argument",
        "imported generic function arity failure",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "argument 1",
        "imported generic function arity failure",
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
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic struct `Box` expects 1 type arguments, found 2",
        "imported generic struct constructor arity",
    );
    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic enum `Option` expects 1 type arguments, found 2",
        "imported generic enum constructor arity",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "field `value` for struct `Box`",
        "imported generic struct constructor arity failure",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "payload for enum variant",
        "imported generic enum constructor arity failure",
    );
}
