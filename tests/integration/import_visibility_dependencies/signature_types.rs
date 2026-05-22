use super::*;

#[test]
fn imported_function_signature_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Point: {
    x: i32
}

pub make_point = () Point {
    Point { x: 109 }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ make_point } = model

main = () i32 {
    point = Point { x: 109 }
    point.x
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct signature dependency type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Point'")
            || message.contains("unknown type `Point`")
            || message.contains("unknown struct `Point`"),
        "expected unimported signature dependency type diagnostic, panic={message}"
    );
}
