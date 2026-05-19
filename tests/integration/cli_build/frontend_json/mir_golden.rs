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

#[test]
fn emit_json_mir_generic_option_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_enum_option.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic Option program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic Option MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("MIR generic Option stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("MIR generic Option stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_generic_option.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_method.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic method input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic method MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("MIR generic method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("MIR generic method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_generic_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_self_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_method_self.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic Self method input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic Self method MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("MIR generic Self method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR generic Self method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_generic_self_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_method_worklist_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_method_worklist.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic method worklist input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic method worklist MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("MIR generic method worklist stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR generic method worklist stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_generic_method_worklist.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_result_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_result_enum_method.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic Result method program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic Result method MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("MIR generic Result method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR generic Result method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_generic_result_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_nested_generic_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/generic_nested_result_enum.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on nested generic Result program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked nested generic Result MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("MIR nested generic Result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR nested generic Result stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/mir_nested_generic_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_behavior_association_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/behavior_json_generic_association.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic behavior association input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic behavior association MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("MIR generic behavior association stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR generic behavior association stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/mir_generic_behavior_association.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_mir_generic_behavior_bound_ufcs_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            fixture("tests/zen/behavior_json_generic_bound_ufcs.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir on generic behavior-bound UFCS input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked generic behavior-bound UFCS MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("MIR generic behavior-bound UFCS stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("MIR generic behavior-bound UFCS stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/mir_generic_behavior_bound_ufcs.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
