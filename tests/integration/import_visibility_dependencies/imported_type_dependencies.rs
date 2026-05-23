use super::*;

#[test]
fn imported_type_impl_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    write_module(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    );

    let model_path = tmp.path().join("model.zen");
    write_module(
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
    );

    let main_path = tmp.path().join("main.zen");
    write_module(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    holder = Holder<i32> { value: 61 }
    holder.get<i32>()
}
"#,
    );

    let message = compile_error_message(
        &main_path,
        "compile_to_c should reject direct source-module imported type use",
    );
    assert_message_contains_any(
        &message,
        &[
            "unknown type symbol 'Holder'",
            "unknown type `Holder`",
            "unknown generic type `Holder`",
            "type `Holder_i32` has no method `get`",
        ],
        "expected unimported helper type or method diagnostic",
    );
}

#[test]
fn imported_generic_function_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    write_module(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    );

    let model_path = tmp.path().join("model.zen");
    write_module(
        &model_path,
        r#"
{ Holder } = helper

pub get_held<T> = (value: T) T {
    holder = Holder<T> { value: value }
    holder.get<T>()
}
"#,
    );

    let main_path = tmp.path().join("main.zen");
    write_module(
        &main_path,
        r#"
{ get_held } = model

main = () i32 {
    holder = Holder<i32> { value: 73 }
    holder.get<i32>()
}
"#,
    );

    let message = compile_error_message(
        &main_path,
        "compile_to_c should reject direct source-module imported type use",
    );
    assert_message_contains_any(
        &message,
        &[
            "unknown type symbol 'Holder'",
            "unknown type `Holder`",
            "unknown generic type `Holder`",
            "type `Holder_i32` has no method `get`",
        ],
        "expected unimported helper type or method diagnostic",
    );
}
