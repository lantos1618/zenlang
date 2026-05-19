use super::assert_invalid_target_yaml;

#[test]
fn emit_json_target_yaml_rejects_unsupported_backend_codegen() {
    assert_invalid_target_yaml(
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
backend:
  codegen: llvm
"#,
        "unsupported backend YAML",
        "target YAML `backend.codegen` supports only `c`",
    );
}

#[test]
fn emit_json_target_yaml_rejects_empty_c_backend_flags() {
    assert_invalid_target_yaml(
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
        "empty C backend flags",
        "target YAML `backend.c_flags` entries cannot be empty",
    );
}

#[test]
fn emit_json_target_yaml_rejects_layout_overrides() {
    assert_invalid_target_yaml(
        r#"
triple: x86_64-unknown-linux-gnu
pointer_width: 64
endianness: little
abi: sysv
overrides:
  i32: i64
"#,
        "layout overrides",
        "target YAML cannot override compiler-owned type layouts",
    );
}
