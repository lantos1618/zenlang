use super::support::{
    assert_diagnostic_code_and_message, assert_no_diagnostic_message, frontend_diagnostics,
    write_tmp_module,
};

#[test]
fn imported_generic_function_inference_conflict_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "helpers.zen",
        r#"
pub choose<T> = (left: T, right: T) T {
    left
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ choose } = helpers

main = () i32 {
    value = choose(1, "bad")
    0
}
"#,
    );
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5000",
        "conflicting inferred type argument `T` for generic function `choose`",
        "imported generic function inference conflict",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "argument 2",
        "imported generic function inference conflict",
    );
}

#[test]
fn imported_generic_method_inference_conflict_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "boxes.zen",
        r#"
pub Box<T>: {
    value: T
}

pub Box.choose<T> = (self: Box<T>, other: T) T {
    self.value
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ Box } = boxes

main = () i32 {
    box = Box<i32> { value: 1 }
    value = box.choose("bad")
    0
}
"#,
    );
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5000",
        "conflicting inferred type argument `T` for generic method `Box.choose`",
        "imported generic method inference conflict",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "argument 2",
        "imported generic method inference conflict",
    );
}

#[test]
fn imported_generic_ufc_explicit_type_arg_arity_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "helpers.zen",
        r#"
pub take_second<T, U> = (first: T, second: U) U {
    second
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ take_second } = helpers

main = () i32 {
    value = 1.take_second<i32>("bad")
    0
}
"#,
    );
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic function `take_second` expects 2 type arguments, found 1",
        "imported generic UFC arity",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "argument 2",
        "imported generic UFC arity failure",
    );
}

#[test]
fn imported_nongeneric_ufc_explicit_type_args_are_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "helpers.zen",
        r#"
pub id_i32 = (value: i32) i32 {
    value
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ id_i32 } = helpers

main = () i32 {
    value = 1.id_i32<i32>()
    0
}
"#,
    );
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5002",
        "non-generic function `id_i32` does not accept type arguments",
        "imported non-generic UFC type-argument",
    );
}

#[test]
fn imported_generic_ufc_behavior_bound_failure_is_error() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "traits.zen",
        r#"
pub Json: behavior {
    encode: (Self) StaticString
}
"#,
    );
    write_tmp_module(
        tmp.path(),
        "helpers.zen",
        r#"
{ Json } = traits

pub as_json<T: Json> = (value: T) StaticString {
    value.encode()
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ as_json } = helpers

Point: {
    x: i32
}

main = () i32 {
    point = Point { x: 1 }
    text = point.as_json()
    0
}
"#,
    );
    let diagnostics = frontend_diagnostics(&main_path);

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E6004",
        "type `Point` does not implement behavior `Json` required by `T`",
        "imported generic UFC behavior bound",
    );
    assert_no_diagnostic_message(
        &diagnostics,
        "has no method `encode`",
        "imported generic UFC behavior-bound failure",
    );
}
