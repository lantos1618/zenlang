use crate::support::*;
use std::path::Path;
use std::process::Command;

#[path = "typed_golden/methods.rs"]
mod methods;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_typed_golden(source: &str, golden: &str, description: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join(source).to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json typed on {description}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked {description} JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("{description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("{description} stdout is JSON: {err}"));
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_typed_generic_type_impl_methods_schema_matches_golden() {
    assert_typed_golden(
        "generic_type_impl_methods.zen",
        "tests/fixtures/ir_json/typed_generic_type_impl_methods.golden.json",
        "generic type impl methods",
    );
}

#[test]
fn emit_json_typed_generic_self_method_schema_matches_golden() {
    assert_typed_golden(
        "generic_method_self.zen",
        "tests/fixtures/ir_json/typed_generic_self_method.golden.json",
        "generic Self method",
    );
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
fn emit_json_typed_generic_ufc_function_schema_matches_golden() {
    assert_typed_golden(
        "generic_ufc_function.zen",
        "tests/fixtures/ir_json/typed_generic_ufc_function.golden.json",
        "generic UFC function",
    );
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
fn emit_json_typed_generic_vec_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_vec.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed on generic Vec input");

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked generic Vec JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("typed generic Vec stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("typed generic Vec stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/typed_generic_vec.golden.json");
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
