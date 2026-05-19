use super::super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_nested_generic_annotation_inner_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "nested_generic_annotation_inner_arity.zen",
        r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

read = (box: Box<Option<i32, StaticString>>) i32 {
    0
}
"#,
        "nested generic annotation inner arity",
        "nested generic annotation inner arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_nested_generic_annotation_inner_arity.golden.json",
    );
}
