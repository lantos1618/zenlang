use super::*;

#[test]
fn imported_type_impl_imported_type_dependencies_are_not_directly_visible() {
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

pub Box<T>: {
    value: T
}

Box.impl = {
    pub get_held<T> = (self: Box<T>) T {
        holder = Holder<T> { value: self.value }
        holder.get<T>()
    }
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
    holder = Holder<i32> { value: 61 }
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
