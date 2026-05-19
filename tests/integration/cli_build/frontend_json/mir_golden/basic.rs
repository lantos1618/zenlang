use super::assert_mir_source_golden;

#[test]
fn emit_json_mir_match_schema_matches_golden() {
    assert_mir_source_golden(
        r#"
Choice:
    Empty,
    Value(i32)

score = (choice: Choice) i32 {
    choice ?
        | Empty { 0 }
        | Value(n) { n }
}

main = () i32 {
    score(Choice.Value(42))
}
"#,
        "mir_match_subject.zen",
        "tests/fixtures/ir_json/mir_match_schema.golden.json",
        "match program",
    );
}

#[test]
fn emit_json_mir_minimal_function_schema_matches_golden() {
    assert_mir_source_golden(
        r#"
main = () i32 {
    value = 40 + 2
    value
}
"#,
        "mir_subject.zen",
        "tests/fixtures/ir_json/mir_minimal_function.golden.json",
        "minimal function",
    );
}
