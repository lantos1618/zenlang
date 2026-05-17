use super::*;

#[test]
fn imported_type_method_worklist_helpers_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to unimported helpers");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'inner'")
            || message.contains("undefined function `inner`"),
        "expected unimported helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_method_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Box<T>: {
    value: T
}

Box.inner<T> = (self: Box<T>) T {
    self.value
}

pub Box.get_inner<T> = (self: Box<T>) T {
    self.inner<T>()
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 47 }
    box.inner<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to unimported methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Box_i32` has no method `inner`")
            || message.contains("type `Box` has no method `inner`"),
        "expected unimported method diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_method_imported_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub inner<T> = (value: T) T {
    value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ inner } = helper

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(59)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to source-module imports");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'inner'")
            || message.contains("undefined function `inner`"),
        "expected unimported helper diagnostic, panic={message}"
    );
}
