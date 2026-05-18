use super::super::*;

#[test]
fn imported_behavior_extends_requires_parent_methods() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString {
        "point"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported inherited behavior requirements");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("implementation of `PrettyJson` is missing required method `encode`"),
        "expected inherited behavior method diagnostic, panic={message}"
    );
}

#[test]
fn imported_behavior_extends_imported_parent_requires_parent_methods() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let base_path = tmp.path().join("base.zen");
    std::fs::write(
        &base_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}
"#,
    )
    .expect("write base module");

    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
{ Json } = base

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString {
        "point"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject inherited imported parent requirements");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("implementation of `PrettyJson` is missing required method `encode`"),
        "expected imported parent behavior method diagnostic, panic={message}"
    );
}

#[test]
fn imported_behavior_extends_requires_transitive_parent_methods() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

pub FancyJson: behavior {
    fancy: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
FancyJson.extends(PrettyJson)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ FancyJson } = traits

Point: {
    x: i32
}

Point.implements(FancyJson) {
    pretty = (value: Point) StaticString {
        "pretty"
    }

    fancy = (value: Point) StaticString {
        "fancy"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path)).expect_err(
        "compile_to_c should reject transitive imported inherited behavior requirements",
    );
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("implementation of `FancyJson` is missing required method `encode`"),
        "expected transitive inherited behavior method diagnostic, panic={message}"
    );
}
