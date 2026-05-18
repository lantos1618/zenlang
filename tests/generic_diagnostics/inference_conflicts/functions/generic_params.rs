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
    box = Box<str> { value: "bad" }
    choose_box(1, box)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_box`: inferred `i32` and `str`"
        )),
        "expected generic function struct inference conflict diagnostic, got {errors:?}"
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
    value = Option<str>.Some("bad")
    choose_option(1, value)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d.message.contains(
            "conflicting inferred type argument `T` for generic function `choose_option`: inferred `i32` and `str`"
        )),
        "expected generic function enum inference conflict diagnostic, got {errors:?}"
    );
}
