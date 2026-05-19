use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "module_graph/ast.rs"]
mod ast;
#[path = "module_graph/symbols.rs"]
mod symbols;

fn write_two_module_project(tmp: &tempfile::TempDir) -> PathBuf {
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(20, 22)
}
"#,
    )
    .expect("write entry module");

    main_path
}

fn emit_json(mode: &str, source_path: &Path, description: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", mode, source_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json {mode} for {description}: {err}"))
}
