use super::*;

#[test]
fn generic_function_explicit_type_arg_arity_does_not_emit_inference_followup() {
    let errors = typecheck_errors(
        r#"
pick<T, U> = (value: T) T {
    value
}

main = () i32 {
    pick<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `pick` expects 2 type arguments, found 1")),
        "expected explicit generic arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("cannot infer type argument")),
        "explicit generic arity failure should not also report inference, got {errors:?}"
    );
}

#[test]
fn generic_method_explicit_type_arg_arity_does_not_emit_inference_followup() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.pick<T, U> = (self: Box, value: T) T {
    value
}

main = () i32 {
    box = Box { value: 1 }
    box.pick<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Box.pick` expects 2 type arguments, found 1")),
        "expected explicit generic method arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("cannot infer type argument")),
        "explicit generic method arity failure should not also report inference, got {errors:?}"
    );
}

#[test]
fn generic_function_explicit_type_arg_arity_does_not_emit_argument_followup() {
    let errors = typecheck_errors(
        r#"
take_second<T, U> = (value: U) U {
    value
}

main = () i32 {
    take_second<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `take_second` expects 2 type arguments, found 1")),
        "expected explicit generic arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 1")),
        "explicit generic arity failure should not also report argument mismatch, got {errors:?}"
    );
}

#[test]
fn generic_method_explicit_type_arg_arity_does_not_emit_argument_followup() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.take_second<T, U> = (self: Box, value: U) U {
    value
}

main = () i32 {
    box = Box { value: 1 }
    box.take_second<i32>(1)
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Box.take_second` expects 2 type arguments, found 1")),
        "expected explicit generic method arity diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().all(|d| !d.message.contains("argument 2")),
        "explicit generic method arity failure should not also report argument mismatch, got {errors:?}"
    );
}
