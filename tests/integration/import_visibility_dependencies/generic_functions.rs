use super::*;

#[test]
fn imported_generic_function_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ Holder } = helper

pub get_held<T> = (value: T) T {
    holder = Holder<T> { value: value }
    holder.get<T>()
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ get_held } = model

main = () i32 {
    holder = Holder<i32> { value: 73 }
    holder.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct source-module imported type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Holder'")
            || message.contains("unknown type `Holder`")
            || message.contains("unknown generic type `Holder`")
            || message.contains("type `Holder_i32` has no method `get`"),
        "expected unimported helper type or method diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_function_transitive_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub middle<T> = (value: T) T {
    inner(value)
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ middle } = helper

pub outer<T> = (value: T) T {
    middle(value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ outer } = model

main = () i32 {
    middle<i32>(89)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct transitive helper calls");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'middle'")
            || message.contains("undefined function `middle`"),
        "expected unimported transitive helper diagnostic, panic={message}"
    );
}
