use std::path::PathBuf;
use std::process::{Command, Output};

pub(super) fn write_single_executable_graph(tmp: &tempfile::TempDir) {
    write_file(
        tmp,
        "build.zen",
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );
    write_file(
        tmp,
        "main.zen",
        r#"
main = () i32 {
    0
}
"#,
    );
}

pub(super) fn write_multiple_executable_graph(tmp: &tempfile::TempDir) {
    write_file(
        tmp,
        "build.zen",
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    );
    write_file(tmp, "app.zen", main_source("0").as_str());
    write_file(tmp, "tool.zen", main_source("0").as_str());
}

pub(super) fn write_dependent_executable_graph(tmp: &tempfile::TempDir) {
    write_file(
        tmp,
        "build.zen",
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    );
    write_file(tmp, "app.zen", main_source("0").as_str());
    write_file(tmp, "tool.zen", main_source("0").as_str());
}

pub(super) fn run_zen_in(tmp: &tempfile::TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen command")
}

pub(super) fn assert_zen_success(args: &[&str], output: &Output) {
    assert!(
        output.status.success(),
        "zen {args:?} failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(super) fn assert_built_binaries_run(paths: &[PathBuf]) {
    for bin_path in paths {
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
        let run = Command::new(bin_path).output().expect("run built binary");
        assert!(
            run.status.success(),
            "built binary {} exited with {}",
            bin_path.display(),
            run.status
        );
    }
}

fn write_file(tmp: &tempfile::TempDir, path: &str, source: &str) {
    std::fs::write(tmp.path().join(path), source).unwrap_or_else(|err| {
        panic!("write {path}: {err}");
    });
}

fn main_source(value: &str) -> String {
    format!(
        r#"
main = () i32 {{
    {value}
}}
"#,
    )
}

pub(super) fn transitive_gated_test_dependency_graph() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen" })
    b.add(Library {
        name: "core",
        exports: ["lib.zen"],
        dependencies: ["unit"],
    })
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");
    tmp
}
