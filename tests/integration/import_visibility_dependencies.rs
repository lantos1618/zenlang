use super::*;

#[path = "import_visibility_dependencies/imported_type_dependencies.rs"]
mod imported_type_dependencies;

fn write_module(path: &std::path::Path, contents: &str) {
    std::fs::write(path, contents).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
}

fn compile_error_message(source_path: &std::path::Path, expectation: &str) -> String {
    let panic = std::panic::catch_unwind(|| compile_to_c(source_path)).expect_err(expectation);

    panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>")
        .to_string()
}

fn assert_message_contains_any(message: &str, expected: &[&str], context: &str) {
    assert!(
        expected.iter().any(|needle| message.contains(needle)),
        "{context}, panic={message}"
    );
}

#[test]
fn imported_generic_function_transitive_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    write_module(
        &helper_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub middle<T> = (value: T) T {
    inner(value)
}
"#,
    );

    let model_path = tmp.path().join("model.zen");
    write_module(
        &model_path,
        r#"
{ middle } = helper

pub outer<T> = (value: T) T {
    middle(value)
}
"#,
    );

    let main_path = tmp.path().join("main.zen");
    write_module(
        &main_path,
        r#"
{ outer } = model

main = () i32 {
    middle<i32>(89)
}
"#,
    );

    let message = compile_error_message(
        &main_path,
        "compile_to_c should reject direct transitive helper calls",
    );
    assert_message_contains_any(
        &message,
        &[
            "unknown value symbol 'middle'",
            "undefined function `middle`",
        ],
        "expected unimported transitive helper diagnostic",
    );
}

#[test]
fn imported_function_signature_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    write_module(
        &model_path,
        r#"
pub Point: {
    x: i32
}

pub make_point = () Point {
    Point { x: 109 }
}
"#,
    );

    let main_path = tmp.path().join("main.zen");
    write_module(
        &main_path,
        r#"
{ make_point } = model

main = () i32 {
    point = Point { x: 109 }
    point.x
}
"#,
    );

    let message = compile_error_message(
        &main_path,
        "compile_to_c should reject direct signature dependency type use",
    );
    assert_message_contains_any(
        &message,
        &[
            "unknown type symbol 'Point'",
            "unknown type `Point`",
            "unknown struct `Point`",
        ],
        "expected unimported signature dependency type diagnostic",
    );
}
