use super::support::{compile_to_c_panic_message, write_tmp_module};

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
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("conflicting inferred type argument `T` for generic function `choose`"),
        "expected imported generic function inference conflict, panic={message}"
    );
    assert!(
        !message.contains("argument 2"),
        "inference conflict should not also report argument mismatch, panic={message}"
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
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("conflicting inferred type argument `T` for generic method `Box.choose`"),
        "expected imported generic method inference conflict, panic={message}"
    );
    assert!(
        !message.contains("argument 2"),
        "inference conflict should not also report argument mismatch, panic={message}"
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
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("generic function `take_second` expects 2 type arguments, found 1"),
        "expected imported generic UFC arity diagnostic, panic={message}"
    );
    assert!(
        !message.contains("argument 2"),
        "imported generic UFC arity failure should not also report argument mismatch, panic={message}"
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
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("non-generic function `id_i32` does not accept type arguments"),
        "expected imported non-generic UFC type-argument diagnostic, panic={message}"
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
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("type `Point` does not implement behavior `Json` required by `T`"),
        "expected imported generic UFC behavior-bound diagnostic, panic={message}"
    );
    assert!(
        !message.contains("has no method `encode`"),
        "bound failure should not also specialize into an unknown-method diagnostic, panic={message}"
    );
}
