use super::*;

#[path = "frontend_diagnostics/behavior_extends/mod.rs"]
mod behavior_extends;

#[test]
fn integration_frontend_helper_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_resolver_ref.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    missing_local
}
"#,
    )
    .expect("write test file");

    let panic = std::panic::catch_unwind(|| compile_to_c(&zen_path))
        .expect_err("compile_to_c should reject resolver errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, panic={message}"
    );
}

#[test]
fn integration_frontend_helper_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported module type errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_enum_method_explicit_type_arg_arity_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let result_path = tmp.path().join("result.zen");
    std::fs::write(
        &result_path,
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
    )
    .expect("write result module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Result } = result

main = () i32 {
    value = Result<i32, str>.Ok(1)
    value.unwrap_or<i32>(0)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported generic method arity errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

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
    let helpers_path = tmp.path().join("helpers.zen");
    std::fs::write(
        &helpers_path,
        r#"
pub take_second<T, U> = (value: U) U {
    value
}
"#,
    )
    .expect("write helpers module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ take_second } = helpers

main = () i32 {
    take_second<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported generic function arity errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

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
fn imported_generic_behavior_requires_missing_impl_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Json } = traits

Point: {
    x: i32
}

Point.requires(Json<str>)

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject missing imported behavior requires impl");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Point` does not implement required behavior `Json_str`"),
        "expected imported generic behavior requires diagnostic, panic={message}"
    );
}
