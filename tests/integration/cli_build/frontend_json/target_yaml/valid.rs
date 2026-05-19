use super::assert_valid_target_yaml;

#[test]
fn emit_json_target_yaml_validates_minimal_target_schema() {
    let json = assert_valid_target_yaml(
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
"#,
        "minimal target YAML",
    );

    assert_eq!(json["format"], "zen.target.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "validated");
    assert_eq!(json["target"]["triple"], "x86_64-unknown-linux-gnu");
    assert_eq!(json["target"]["pointer_width"], 64);
    assert_eq!(json["target"]["endianness"], "little");
    assert_eq!(json["target"]["abi"], "sysv");
}

#[test]
fn emit_json_target_yaml_validates_backend_schema() {
    let json = assert_valid_target_yaml(
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
backend:
  codegen: c
  c_compiler: cc
"#,
        "backend target YAML",
    );

    assert_eq!(json["format"], "zen.target.v0");
    assert_eq!(json["semantic_status"], "validated");
    assert_eq!(json["target"]["backend"]["codegen"], "c");
    assert_eq!(json["target"]["backend"]["c_compiler"], "cc");
}

#[test]
fn emit_json_target_yaml_validates_c_backend_flags() {
    let json = assert_valid_target_yaml(
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
        "C backend flags target YAML",
    );

    assert_eq!(json["target"]["backend"]["codegen"], "c");
    assert_eq!(json["target"]["backend"]["c_flags"][0], "-std=c11");
    assert_eq!(json["target"]["backend"]["c_flags"][1], "-Wall");
}
