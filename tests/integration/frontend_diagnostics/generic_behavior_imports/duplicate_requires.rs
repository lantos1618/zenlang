use super::super::*;

#[test]
fn imported_duplicate_generic_behavior_requires_is_error() {
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

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString {
        "point"
    }
}

Point.requires(Json<StaticString>)
Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject duplicate imported behavior requires");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("duplicate required behavior `Json<StaticString>` for `Point`"),
        "expected imported duplicate behavior requires diagnostic, panic={message}"
    );
}
