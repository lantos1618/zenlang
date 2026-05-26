use super::super::super::*;

#[test]
fn generic_method_inference_conflict_through_generic_struct_type_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Holder: {
    value: i32
}

Holder.choose_box<T> = (self: Holder, left: T, box: Box<T>) T {
    left
}

main = () i32 {
    holder = Holder { value: 0 }
    box = Box<StaticString> { value: "bad" }
    holder.choose_box(1, box)
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "method",
        "Holder.choose_box",
        "T",
        "i32",
        "StaticString",
        "generic method struct inference conflict",
    );
}

#[test]
fn generic_method_inference_conflict_through_generic_enum_type_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

Holder: {
    value: i32
}

Holder.choose_option<T> = (self: Holder, left: T, value: Option<T>) T {
    left
}

main = () i32 {
    holder = Holder { value: 0 }
    value = Option<StaticString>.Some("bad")
    holder.choose_option(1, value)
}
"#,
    );

    assert_inference_conflict(
        &errors,
        "method",
        "Holder.choose_option",
        "T",
        "i32",
        "StaticString",
        "generic method enum inference conflict",
    );
}
