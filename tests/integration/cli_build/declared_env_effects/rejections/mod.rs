#[path = "build.rs"]
mod build;
#[path = "build_graph.rs"]
mod build_graph;
#[path = "check.rs"]
mod check;
#[path = "direct_file.rs"]
mod direct_file;
#[path = "emit.rs"]
mod emit;
#[path = "test_command.rs"]
mod test_command;

use std::process::Command;

use super::*;

fn assert_env_read_without_fallback_before_unselected_targets(args: &[&str], build_message: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen executable build graph command");

    assert_env_read_without_fallback_failed(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("missing_unit.zen"),
        "host-effect validation should run before unrelated test source handling, stderr={stderr}"
    );
    assert!(!tmp.path().join("build").exists(), "{build_message}");
}
