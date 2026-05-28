use std::path::Path;

use super::super::support::{assert_zen_success, run_zen, write_file};

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_target_yaml_backend_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(
        &tmp,
        "target.yaml",
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
    );

    let yaml_path = tmp.path().join("target.yaml");
    let args = ["emit-json", "target-yaml", yaml_path.to_str().unwrap()];
    let output = run_zen(&args);

    assert_zen_success(&args, &output);

    let actual = String::from_utf8(output.stdout).expect("target-yaml stdout is UTF-8");
    serde_json::from_str::<serde_json::Value>(&actual).expect("target-yaml stdout is JSON");
    let expected_path = fixture("tests/fixtures/ir_json/target_yaml_backend.golden.json");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
