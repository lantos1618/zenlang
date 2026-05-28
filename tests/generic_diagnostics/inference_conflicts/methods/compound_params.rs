use super::super::super::*;

#[test]
fn generic_method_inference_conflicts_through_compound_params_are_errors() {
    for (method, parameter, setup, argument, context) in [
        (
            "choose_with",
            "mapper: (T) T",
            r#"    mapper = (value: StaticString) StaticString {
        value
    }"#,
            "mapper",
            "generic method function-type inference conflict",
        ),
        (
            "choose_array",
            "items: [T; 1]",
            r#"    items = ["bad"]"#,
            "items",
            "generic method array-type inference conflict",
        ),
        (
            "choose_raw",
            "ptr: RawPtr<T>",
            r#"    ptr = cast("bad", RawPtr<StaticString>)"#,
            "ptr",
            "generic method raw-pointer inference conflict",
        ),
        (
            "choose_ptr",
            "ptr: Ptr<T>",
            r#"    ptr = cast("bad", Ptr<StaticString>)"#,
            "ptr",
            "generic method pointer inference conflict",
        ),
        (
            "choose_mut_ptr",
            "ptr: MutPtr<T>",
            r#"    ptr = cast("bad", MutPtr<StaticString>)"#,
            "ptr",
            "generic method mutable pointer inference conflict",
        ),
        (
            "choose_slice",
            "items: Slice<T>",
            r#"    items = cast("bad", Slice<StaticString>)"#,
            "items",
            "generic method slice inference conflict",
        ),
    ] {
        assert_compound_param_conflict(method, parameter, setup, argument, context);
    }
}

fn assert_compound_param_conflict(
    method: &str,
    parameter: &str,
    setup: &str,
    argument: &str,
    context: &str,
) {
    let errors = typecheck_errors(&format!(
        r#"
Box<T>: {{
    value: T
}}

Box.{method}<T> = (self: Box<T>, {parameter}) T {{
    self.value
}}

main = () i32 {{
    box = Box<i32> {{ value: 1 }}
{setup}
    box.{method}({argument})
}}
"#,
    ));

    assert_inference_conflict(
        &errors,
        "method",
        &format!("Box.{method}"),
        "T",
        "i32",
        "StaticString",
        context,
    );
}
