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

    assert_generic_arity_diagnostic(&errors, "function", "pick", 2, 1, "generic function arity");
    assert_no_diagnostic_message(&errors, "cannot infer type argument", "function arity");
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

    assert_generic_arity_diagnostic(&errors, "method", "Box.pick", 2, 1, "generic method arity");
    assert_no_diagnostic_message(&errors, "cannot infer type argument", "method arity");
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

    assert_generic_arity_diagnostic(
        &errors,
        "function",
        "take_second",
        2,
        1,
        "generic function arity",
    );
    assert_no_diagnostic_message(&errors, "argument 1", "function arity");
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

    assert_generic_arity_diagnostic(
        &errors,
        "method",
        "Box.take_second",
        2,
        1,
        "generic method arity",
    );
    assert_no_diagnostic_message(&errors, "argument 2", "method arity");
}
