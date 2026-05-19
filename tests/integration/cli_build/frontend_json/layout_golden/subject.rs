use super::assert_layout_matches_fixture;
use super::write_subject;

#[test]
fn emit_json_layout_compound_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(
        &tmp,
        "compound_layout_subject.zen",
        r#"
Handles: {
    ptr: Ptr<i32>,
    raw: RawPtr<i32>,
    slice: Slice<i32>,
    fixed: [i32; 4],
}

Choice:
    Empty,
    WithPayload(Handles)

main = () i32 { 0 }
"#,
    );

    assert_layout_matches_fixture(
        &zen_path,
        "compound program input",
        "tests/fixtures/ir_json/layout_compound.golden.json",
    );
}

#[test]
fn emit_json_layout_basic_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(
        &tmp,
        "layout_subject.zen",
        r#"
Point: {
    x: i32,
    label: StaticString
}

main = () i32 { 0 }
"#,
    );

    assert_layout_matches_fixture(
        &zen_path,
        "program input",
        "tests/fixtures/ir_json/layout_basic.golden.json",
    );
}
