use std::process::Command;

#[test]
fn emit_json_target_yaml_validates_minimal_target_schema() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        output.status.success(),
        "zen emit-json target-yaml should validate target YAML: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("target-yaml stdout is json");
    assert_eq!(json["format"], "zen.target.v0");
    assert_eq!(json["semantic_status"], "validated");
    assert_eq!(json["target"]["triple"], "x86_64-unknown-linux-gnu");
    assert_eq!(json["target"]["pointer_width"], 64);
    assert_eq!(json["target"]["endianness"], "little");
    assert_eq!(json["target"]["abi"], "sysv");
}

#[test]
fn emit_json_target_yaml_validates_backend_schema() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
backend:
  codegen: c
  c_compiler: cc
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        output.status.success(),
        "zen emit-json target-yaml should validate backend YAML: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("target-yaml stdout is json");
    assert_eq!(json["format"], "zen.target.v0");
    assert_eq!(json["semantic_status"], "validated");
    assert_eq!(json["target"]["backend"]["codegen"], "c");
    assert_eq!(json["target"]["backend"]["c_compiler"], "cc");
}

#[test]
fn emit_json_target_yaml_validates_c_backend_flags() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
backend:
  codegen: c
  c_compiler: cc
  c_flags:
    - -std=c11
    - -Wall
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        output.status.success(),
        "zen emit-json target-yaml should validate C backend flags: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("target-yaml stdout is json");
    assert_eq!(json["target"]["backend"]["codegen"], "c");
    assert_eq!(json["target"]["backend"]["c_flags"][0], "-std=c11");
    assert_eq!(json["target"]["backend"]["c_flags"][1], "-Wall");
}

#[test]
fn emit_json_target_yaml_rejects_unsupported_backend_codegen() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
backend:
  codegen: llvm
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        !output.status.success(),
        "zen emit-json target-yaml should reject unsupported backend YAML: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "invalid target-yaml should not emit target JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("target YAML `backend.codegen` supports only `c`"),
        "expected target YAML backend diagnostic, stderr={stderr}"
    );
}

#[test]
fn emit_json_target_yaml_rejects_empty_c_backend_flags() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
backend:
  codegen: c
  c_flags:
    - -std=c11
    - ""
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        !output.status.success(),
        "zen emit-json target-yaml should reject empty C backend flags: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "invalid target-yaml should not emit target JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("target YAML `backend.c_flags` entries cannot be empty"),
        "expected target YAML c_flags diagnostic, stderr={stderr}"
    );
}

#[test]
fn emit_json_target_yaml_rejects_layout_overrides() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
overrides:
  i32: i64
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        !output.status.success(),
        "zen emit-json target-yaml should reject layout overrides: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "invalid target-yaml should not emit target JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("target YAML cannot override compiler-owned type layouts"),
        "expected target YAML layout override diagnostic, stderr={stderr}"
    );
}
