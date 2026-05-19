use std::path::Path;
use std::process::Command;

#[path = "hir_golden/generic_enums.rs"]
mod generic_enums;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_hir_declaration_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("hir_declarations_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Pair: {
    left: i32,
    right: i32,
}

MaybePair:
    None,
    Some(Pair)

threshold ::= 10

choose = (candidate: Pair, enabled: bool) MaybePair {
    enabled ?
        | true { MaybePair.Some(candidate) }
        | false { MaybePair.None }
}

main = () i32 { 0 }
"#,
    )
    .expect("write HIR declarations subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "hir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json hir on declaration-rich program input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked declaration-rich HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("HIR declarations stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("HIR declarations stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_declarations.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_vec_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_vec.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic Vec input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic Vec HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("HIR generic Vec stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("HIR generic Vec stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_vec.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_method.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic method input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic method HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("HIR generic method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("HIR generic method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_type_impl_methods_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_type_impl_methods.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic type impl methods input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic type impl methods HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR generic type impl methods stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic type impl methods stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_type_impl_methods.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_self_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_method_self.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic Self method input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic Self method HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("HIR generic Self method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic Self method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_self_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_method_worklist_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_method_worklist.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic method worklist input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic method worklist HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR generic method worklist stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic method worklist stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_method_worklist.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_function_worklist_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_worklist.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic function worklist input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic function worklist HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR generic function worklist stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic function worklist stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_function_worklist.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_result_method_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_result_enum_method.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic Result method program input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic Result method HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR generic Result method stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic Result method stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_generic_result_method.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_nested_generic_result_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/generic_nested_result_enum.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on nested generic Result program input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked nested generic Result HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR nested generic Result stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR nested generic Result stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/hir_nested_generic_result.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_behavior_association_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/behavior_json_generic_association.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic behavior association input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic behavior association HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR generic behavior association stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic behavior association stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/hir_generic_behavior_association.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_hir_generic_behavior_bound_ufcs_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            fixture("tests/zen/behavior_json_generic_bound_ufcs.zen")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir on generic behavior-bound UFCS input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked generic behavior-bound UFCS HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual =
        String::from_utf8(output.stdout).expect("HIR generic behavior-bound UFCS stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual)
        .expect("HIR generic behavior-bound UFCS stdout is JSON");
    let expected_path =
        fixture("tests/fixtures/ir_json/hir_generic_behavior_bound_ufcs.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
