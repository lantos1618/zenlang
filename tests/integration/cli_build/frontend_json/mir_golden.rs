use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_mir_match_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("mir_match_subject.zen");
    std::fs::write(
        &zen_path,
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
    )
    .expect("write MIR match subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json mir on match program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked MIR match JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("MIR match stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("MIR match stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_match_schema.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_minimal_function_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("mir_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    value = 40 + 2
    value
}
"#,
    )
    .expect("write MIR subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json mir on program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked minimal MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("MIR minimal stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("MIR minimal stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_minimal_function.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_result_enum.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic Result program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic Result MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("MIR generic Result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("MIR generic Result stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_generic_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
