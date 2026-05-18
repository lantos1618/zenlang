use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_function_type_arg_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_function_type_arg_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T {
    value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = identity<Box<i32, StaticString>>(box)
    0
}
"#,
        "generic function type-argument annotation arity",
        "generic function type-argument annotation arity diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_generic_function_type_arg_annotation_arity.golden.json",
    );
}
