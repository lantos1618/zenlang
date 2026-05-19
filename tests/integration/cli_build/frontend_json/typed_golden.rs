use crate::support::*;
use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_typed_generic_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_method.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed");

    assert!(
        output.status.success(),
        "zen emit-json typed failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("typed stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("typed stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_method_worklist_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_method_worklist.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic method worklist input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic method worklist JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("typed generic method worklist stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic method worklist stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_method_worklist.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_method_nested_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_method_nested_result.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic method nested Result input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic method nested Result JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .expect("typed generic method nested Result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic method nested Result stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/typed_generic_method_nested_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_ufc_dedup_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_ufc_dedup.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic UFC dedup input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic UFC dedup JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("typed generic UFC dedup stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic UFC dedup stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_ufc_dedup.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_worklist_dedup_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_worklist_dedup.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic worklist dedup input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic worklist dedup JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("typed generic worklist dedup stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic worklist dedup stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_worklist_dedup.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_option_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_enum_option.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic Option program input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic Option JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("typed generic Option stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic Option stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_option.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_option_multi_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_enum_multi_specialization.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic Option multi-specialization input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic Option multi-specialization JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("typed generic Option multi stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic Option multi stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_option_multi.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_result_enum.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic Result program input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic Result JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("typed generic Result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic Result stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_result_multi_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_result_enum_multi_specialization.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic Result multi-specialization input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic Result multi-specialization JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("typed generic Result multi stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic Result multi stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_result_multi.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_result_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_result_enum_method.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic Result method input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic Result method JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("typed generic Result method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic Result method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_result_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_nested_generic_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("generic_nested_result_enum.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on nested generic Result program input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked nested generic Result JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("typed nested generic Result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed nested generic Result stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_nested_generic_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_behavior_association_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("behavior_json_generic_association.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic behavior association input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic behavior association JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .expect("typed generic behavior association stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic behavior association stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/typed_generic_behavior_association.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_behavior_bound_ufcs_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir()
                .join("behavior_json_generic_bound_ufcs.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic behavior-bound UFCS input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic behavior-bound UFCS JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .expect("typed generic behavior-bound UFCS stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("typed generic behavior-bound UFCS stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/typed_generic_behavior_bound_ufcs.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
