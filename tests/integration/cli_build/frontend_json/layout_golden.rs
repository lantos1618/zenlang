use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_layout_compound_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("compound_layout_subject.zen");
    std::fs::write(
        &zen_path,
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
    )
    .expect("write compound layout subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "layout", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json layout on compound program input");

    assert!(
        output.status.success(),
        "zen emit-json layout should emit checked compound layout JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("layout stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("layout stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/layout_compound.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_layout_basic_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("layout_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Point: {
    x: i32,
    label: StaticString
}

main = () i32 { 0 }
"#,
    )
    .expect("write layout subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "layout", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json layout on program input");

    assert!(
        output.status.success(),
        "zen emit-json layout should emit checked layout JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("layout stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("layout stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/layout_basic.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
