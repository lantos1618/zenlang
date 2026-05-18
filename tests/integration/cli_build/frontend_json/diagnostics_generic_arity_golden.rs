use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_diagnostics_generic_function_arity_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_function_arity.zen");
    std::fs::write(
        &zen_path,
        r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    identity<i32, StaticString>(1)
}
"#,
    )
    .expect("write generic function arity source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic function arity: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "generic function arity diagnostics should not emit inference or argument followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_function_arity.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_struct_constructor_arity_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_struct_constructor_arity.zen");
    std::fs::write(
        &zen_path,
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    boxed = Box<i32, StaticString> { value: 1 }
    boxed.value
}
"#,
    )
    .expect("write generic struct constructor arity source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic struct constructor arity: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "generic struct constructor arity diagnostics should not emit field mismatch followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_struct_constructor_arity.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_enum_constructor_arity_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_enum_constructor_arity.zen");
    std::fs::write(
        &zen_path,
        r#"
Option<T>:
    Some(T),
    None

main = () i32 {
    value = Option<i32, StaticString>.Some(1)
    0
}
"#,
    )
    .expect("write generic enum constructor arity source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic enum constructor arity: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "generic enum constructor arity diagnostics should not emit payload mismatch followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_enum_constructor_arity.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_struct_annotation_arity_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_struct_annotation_arity.zen");
    std::fs::write(
        &zen_path,
        r#"
Box<T>: {
    value: T
}

read = (box: Box<i32, StaticString>) i32 {
    0
}
"#,
    )
    .expect("write generic struct annotation arity source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic struct annotation arity: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "generic struct annotation arity diagnostics should not emit dependent-use followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_struct_annotation_arity.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_enum_annotation_arity_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_enum_annotation_arity.zen");
    std::fs::write(
        &zen_path,
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option<i32, StaticString>) i32 {
    0
}
"#,
    )
    .expect("write generic enum annotation arity source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic enum annotation arity: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "generic enum annotation arity diagnostics should not emit dependent-use followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_enum_annotation_arity.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_struct_annotation_missing_args_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp
        .path()
        .join("generic_struct_annotation_missing_args.zen");
    std::fs::write(
        &zen_path,
        r#"
Box<T>: {
    value: T
}

read = (box: Box) i32 {
    0
}
"#,
    )
    .expect("write generic struct annotation missing args source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic struct annotation missing args: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "generic struct annotation missing-args diagnostics should not emit dependent-use followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path = fixture(
        "tests/fixtures/ir_json/diagnostics_generic_struct_annotation_missing_args.golden.json",
    );
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}
