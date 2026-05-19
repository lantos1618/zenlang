use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn write_subject(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(20, 22)
}
"#,
    )
    .expect("write entry module");

    main_path
}

fn normalized_module_graph_json(mode: &str) -> String {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = write_subject(&tmp);
    normalized_json_for_path(mode, &main_path)
}

fn normalized_json_for_path(mode: &str, path: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", mode, path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json {mode}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json {mode} failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut json: Value =
        serde_json::from_slice(&output.stdout).expect("module graph JSON stdout is JSON");
    for module in json["modules"]
        .as_array_mut()
        .expect("module graph modules array")
    {
        let path = module["canonical_path"]
            .as_str()
            .expect("module canonical path");
        module["canonical_path"] = Path::new(path)
            .file_name()
            .expect("module file name")
            .to_string_lossy()
            .into_owned()
            .into();
    }

    serde_json::to_string_pretty(&json).expect("serialize normalized module graph JSON")
}

fn normalized_fixture(path: &str) -> String {
    let expected_path = fixture(path);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));
    let json: Value = serde_json::from_str(&expected)
        .unwrap_or_else(|err| panic!("parse {}: {err}", expected_path.display()));
    serde_json::to_string_pretty(&json).expect("serialize normalized fixture JSON")
}

#[test]
fn emit_json_ast_module_graph_schema_matches_golden() {
    let actual = normalized_module_graph_json("ast");
    let expected = normalized_fixture("tests/fixtures/ir_json/ast_module_graph.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_module_graph_schema_matches_golden() {
    let actual = normalized_module_graph_json("symbols");
    let expected = normalized_fixture("tests/fixtures/ir_json/symbols_module_graph.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_method_schema_matches_golden() {
    let actual = normalized_json_for_path("symbols", &fixture("tests/zen/generic_method.zen"));
    let expected = normalized_fixture("tests/fixtures/ir_json/symbols_generic_method.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_option_schema_matches_golden() {
    let actual = normalized_json_for_path("symbols", &fixture("tests/zen/generic_enum_option.zen"));
    let expected = normalized_fixture("tests/fixtures/ir_json/symbols_generic_option.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_result_schema_matches_golden() {
    let actual = normalized_json_for_path("symbols", &fixture("tests/zen/generic_result_enum.zen"));
    let expected = normalized_fixture("tests/fixtures/ir_json/symbols_generic_result.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_result_method_schema_matches_golden() {
    let actual = normalized_json_for_path(
        "symbols",
        &fixture("tests/zen/generic_result_enum_method.zen"),
    );
    let expected =
        normalized_fixture("tests/fixtures/ir_json/symbols_generic_result_method.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_type_impl_methods_schema_matches_golden() {
    let actual = normalized_json_for_path(
        "symbols",
        &fixture("tests/zen/generic_type_impl_methods.zen"),
    );
    let expected =
        normalized_fixture("tests/fixtures/ir_json/symbols_generic_type_impl_methods.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_self_method_schema_matches_golden() {
    let actual = normalized_json_for_path("symbols", &fixture("tests/zen/generic_method_self.zen"));
    let expected =
        normalized_fixture("tests/fixtures/ir_json/symbols_generic_self_method.golden.json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_behavior_association_schema_matches_golden() {
    let actual = normalized_json_for_path(
        "symbols",
        &fixture("tests/zen/behavior_json_generic_association.zen"),
    );
    let expected = normalized_fixture(
        "tests/fixtures/ir_json/symbols_generic_behavior_association.golden.json",
    );

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_symbols_generic_behavior_bound_ufcs_schema_matches_golden() {
    let actual = normalized_json_for_path(
        "symbols",
        &fixture("tests/zen/behavior_json_generic_bound_ufcs.zen"),
    );
    let expected = normalized_fixture(
        "tests/fixtures/ir_json/symbols_generic_behavior_bound_ufcs.golden.json",
    );

    assert_eq!(actual.trim(), expected.trim());
}
