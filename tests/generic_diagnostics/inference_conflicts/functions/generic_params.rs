use super::super::super::*;

#[test]
fn generic_function_inference_conflict_through_generic_struct_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

choose_box<T> = (left: T, box: Box<T>) T {
    left
}

main = () i32 {
    box = Box<StaticString> { value: "bad" }
    choose_box(1, box)
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "function",
        "choose_box",
        "T",
        "i32",
        "StaticString",
        "generic function struct inference conflict",
    );
}

#[test]
fn generic_function_inference_conflict_through_generic_enum_type_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

choose_option<T> = (left: T, value: Option<T>) T {
    left
}

main = () i32 {
    value = Option<StaticString>.Some("bad")
    choose_option(1, value)
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "function",
        "choose_option",
        "T",
        "i32",
        "StaticString",
        "generic function enum inference conflict",
    );
}
