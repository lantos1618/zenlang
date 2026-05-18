use super::*;

#[test]
fn imported_private_type_impl_methods_are_not_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Box<T>: {
    value: T
}

Box.impl = {
    get<T> = (self: Box<T>) T {
        self.value
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
    box = Box<i32> { value: 34 }
    box.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject private imported impl methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Box_i32` has no method `get`"),
        "expected private imported impl method diagnostic, panic={message}"
    );
}

#[test]
fn imported_private_behavior_impl_methods_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
Hidden: behavior {
    reveal: (Self) StaticString
}

pub Point: {
    x: i32
}

Point.implements(Hidden) {
    reveal = (value: Point) StaticString {
        "hidden"
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Point } = model

main = () i32 {
    point = Point { x: 34 }
    point.reveal()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject private imported behavior impl methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Point` has no method `reveal`"),
        "expected private imported behavior impl method diagnostic, panic={message}"
    );
}
