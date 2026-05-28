use super::{parse_program, BuildGraph};

const READ_ENV_JSON: &str = r#""kind":"read_env","value":"ZEN_STD""#;
const READ_FILE_JSON: &str = r#""kind":"read_file","value":"build.targets""#;

#[test]
fn build_program_lowering_rejects_undeclared_env_reads() {
    assert_host_effect_error(
        r#"std_path = b.os.env("ZEN_STD")"#,
        "undeclared host effect: read env `ZEN_STD`",
    );
}

#[test]
fn build_program_lowering_accepts_declared_env_reads() {
    for fallback in [".Err", "_", "err"] {
        let statement = env_read_with_fallback(fallback);
        assert_declares_host_effect(&statement, READ_ENV_JSON);
    }
}

#[test]
fn build_program_lowering_rejects_env_read_without_fallback() {
    assert_host_effect_error(
        r#"std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }"#,
        "undeclared host effect: read env `ZEN_STD`",
    );
}

#[test]
fn build_program_lowering_accepts_declared_file_reads() {
    for fallback in [".Err", "_", "err"] {
        let statement = file_read_with_fallback(fallback);
        assert_declares_host_effect(&statement, READ_FILE_JSON);
    }
}

#[test]
fn build_program_lowering_rejects_undeclared_file_reads() {
    assert_host_effect_error(
        r#"manifest = b.os.read_file("build.targets")"#,
        "undeclared host effect: read file `build.targets`",
    );
}

#[test]
fn build_program_lowering_rejects_file_read_without_fallback() {
    assert_host_effect_error(
        r#"manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }"#,
        "undeclared host effect: read file `build.targets`",
    );
}

fn env_read_with_fallback(fallback: &str) -> String {
    format!(
        r#"std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        | {fallback} {{ "default" }}"#
    )
}

fn file_read_with_fallback(fallback: &str) -> String {
    format!(
        r#"manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        | {fallback} {{ "default" }}"#
    )
}

fn assert_declares_host_effect(statement: &str, expected_json: &str) {
    let json = lower_build_program(statement)
        .expect("lower build graph")
        .canonical_json()
        .expect("build graph json");

    assert!(
        json.contains(expected_json),
        "expected declared host effect in graph json, json={json}"
    );
}

fn assert_host_effect_error(statement: &str, expected: &str) {
    let err = lower_build_program(statement).expect_err("host effect should fail");
    assert_eq!(err.to_string(), expected);
}

fn lower_build_program(statement: &str) -> Result<BuildGraph, zen::build_graph::BuildGraphError> {
    BuildGraph::from_build_program(&parse_program(&format!(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    {statement}
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})
    .Ok(b.config())
}}
"#
    )))
}
