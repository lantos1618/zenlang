use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_diagnostics_removed_return_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("return_keyword.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    return 1
}
"#,
    )
    .expect("write removed return source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on removed return syntax: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("diagnostics stdout is JSON");
    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path = fixture("tests/fixtures/ir_json/diagnostics_return.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_behavior_derive_gate_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("derive_gate.zen");
    std::fs::write(
        &zen_path,
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.derive(Json)
"#,
    )
    .expect("write gated derive source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on gated derive association: stdout={}, stderr={}",
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
        "gated derive diagnostics should emit one feature-gate diagnostic: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_behavior_derive_gate.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_association_gate_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_association_gate.zen");
    std::fs::write(
        &zen_path,
        r#"
Box<T>: {
    value: T
}

Json<T>: behavior {
    to_json: (T) StaticString
}

Box<T>.derive(Json<T>)
"#,
    )
    .expect("write gated generic association source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on gated generic association: stdout={}, stderr={}",
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
        "gated generic association diagnostics should emit one feature-gate diagnostic: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_association_gate.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_behavior_overlap_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_behavior_overlap.zen");
    std::fs::write(
        &zen_path,
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    )
    .expect("write generic behavior overlap source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on overlapping generic behavior impls: stdout={}, stderr={}",
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
        "generic behavior overlap should emit one coherence diagnostic: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_behavior_overlap.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
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
fn emit_json_diagnostics_generic_result_method_arity_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_result_method_arity.zen");
    std::fs::write(
        &zen_path,
        r#"
Result<T, E>:
    Ok(T),
    Err(E)

Result.unwrap_or<T, E> = (self: Self, fallback: T) T {
    self ?
        | Ok(value) { value }
        | Err(_) { fallback }
}

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.unwrap_or<i32>(0)
}
"#,
    )
    .expect("write generic method arity source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic method arity: stdout={}, stderr={}",
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
        "generic arity diagnostics should not emit inference or argument followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_result_method_arity.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_result_method_bound_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_result_method_bound.zen");
    std::fs::write(
        &zen_path,
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}

Result<T, E>:
    Ok(T),
    Err(E)

Result.map<T, E, U: Json> = (self: Self, fallback: U) U {
    fallback.encode()
    fallback
}

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    point = Point { x: 1 }
    bad = value.map(point)
    0
}
"#,
    )
    .expect("write generic method bound source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic method bound: stdout={}, stderr={}",
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
        "generic bound diagnostics should not emit method-body followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_result_method_bound.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_generic_result_method_inference_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("generic_result_method_inference.zen");
    std::fs::write(
        &zen_path,
        r#"
Result<T, E>:
    Ok(T),
    Err(E)

Result.unwrap_or<T, E> = (self: Self, fallback: T) T {
    self ?
        | Ok(value) { value }
        | Err(_) { fallback }
}

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.unwrap_or("bad")
}
"#,
    )
    .expect("write generic method inference source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on generic method inference: stdout={}, stderr={}",
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
        "generic inference diagnostics should not emit argument or return followups: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path =
        fixture("tests/fixtures/ir_json/diagnostics_generic_result_method_inference.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}
