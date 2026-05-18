use super::*;

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

#[test]
fn imported_generic_behavior_impl_type_arg_arity_is_error() {
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

Point.implements(Json) {
    encode = (value: Point) str {
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
        .expect_err("compile_to_c should reject imported behavior impl arity errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("generic behavior `Json` expects 1 type arguments, found 0"),
        "expected imported behavior impl arity diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_behavior_extends_type_arg_arity_is_error() {
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

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported behavior extends arity errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("generic behavior `Json` expects 1 type arguments, found 0"),
        "expected imported behavior extends arity diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_behavior_requires_type_arg_arity_is_error() {
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

Point.requires(Json<i32, str>)

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported behavior requires arity errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("generic behavior `Json` expects 1 type arguments, found 2"),
        "expected imported behavior requires arity diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_behavior_bound_type_arg_arity_is_error() {
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

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported generic behavior bound arity errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("generic behavior `Json` expects 1 type arguments, found 0"),
        "expected imported generic behavior bound arity diagnostic, panic={message}"
    );
}

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

Point.implements(Json<str>) {
    encode = (value: Point) str {
        "point"
    }
}

Point.requires(Json<str>)
Point.requires(Json<str>)

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
        message.contains("duplicate required behavior `Json<str>` for `Point`"),
        "expected imported duplicate behavior requires diagnostic, panic={message}"
    );
}
