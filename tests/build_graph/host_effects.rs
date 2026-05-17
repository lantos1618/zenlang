use super::{parse_program, BuildGraph};

#[test]
fn build_program_lowering_rejects_undeclared_env_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("undeclared build.zen env read should fail");

    assert_eq!(
        err.to_string(),
        "undeclared host effect: read env `ZEN_STD`"
    );
}

#[test]
fn build_program_lowering_accepts_declared_env_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
        | .Err { "default" }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");
    let json = graph.canonical_json().expect("build graph json");

    assert!(
        json.contains(r#""kind":"read_env","value":"ZEN_STD""#),
        "expected read-env host effect in graph json, json={json}"
    );
}

#[test]
fn build_program_lowering_accepts_declared_file_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
        | .Err { "default" }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");
    let json = graph.canonical_json().expect("build graph json");

    assert!(
        json.contains(r#""kind":"read_file","value":"build.targets""#),
        "expected read-file host effect in graph json, json={json}"
    );
}

#[test]
fn build_program_lowering_rejects_undeclared_file_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("undeclared build.zen file read should fail");

    assert_eq!(
        err.to_string(),
        "undeclared host effect: read file `build.targets`"
    );
}
