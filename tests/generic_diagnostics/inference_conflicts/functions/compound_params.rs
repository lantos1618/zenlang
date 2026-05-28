use super::super::super::*;

#[test]
fn generic_function_inference_conflicts_through_compound_params_are_errors() {
    for (callee, parameter, setup, argument, context) in [
        (
            "choose_with",
            "mapper: (T) T",
            r#"    mapper = (value: StaticString) StaticString {
        value
    }"#,
            "mapper",
            "generic function function-type inference conflict",
        ),
        (
            "choose_array",
            "items: [T; 1]",
            r#"    items = ["bad"]"#,
            "items",
            "generic function array-type inference conflict",
        ),
        (
            "choose_raw",
            "ptr: RawPtr<T>",
            r#"    ptr = cast("bad", RawPtr<StaticString>)"#,
            "ptr",
            "generic function raw-pointer inference conflict",
        ),
        (
            "choose_ptr",
            "ptr: Ptr<T>",
            r#"    ptr = cast("bad", Ptr<StaticString>)"#,
            "ptr",
            "generic function pointer inference conflict",
        ),
        (
            "choose_mut_ptr",
            "ptr: MutPtr<T>",
            r#"    ptr = cast("bad", MutPtr<StaticString>)"#,
            "ptr",
            "generic function mutable pointer inference conflict",
        ),
        (
            "choose_slice",
            "items: Slice<T>",
            r#"    items = cast("bad", Slice<StaticString>)"#,
            "items",
            "generic function slice inference conflict",
        ),
    ] {
        assert_compound_param_conflict(callee, parameter, setup, argument, context);
    }
}

fn assert_compound_param_conflict(
    callee: &str,
    parameter: &str,
    setup: &str,
    argument: &str,
    context: &str,
) {
    let errors = typecheck_errors(&format!(
        r#"
{callee}<T> = (left: T, {parameter}) T {{
    left
}}

main = () i32 {{
{setup}
    {callee}(1, {argument})
}}
"#,
    ));

    assert_inference_conflict(
        &errors,
        "function",
        callee,
        "T",
        "i32",
        "StaticString",
        context,
    );
}
