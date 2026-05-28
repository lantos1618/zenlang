use std::path::{Path, PathBuf};

use super::super::support::run_zen;
use super::golden_support::write_subject;
mod ast;
mod symbols;

fn write_two_module_project(tmp: &tempfile::TempDir) -> PathBuf {
    write_subject(
        tmp,
        "math.zen",
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}
"#,
    );

    write_subject(
        tmp,
        "main.zen",
        r#"
{ add } = math

main = () i32 {
    add(20, 22)
}
"#,
    )
}

fn emit_json(mode: &str, source_path: &Path, _description: &str) -> std::process::Output {
    run_zen(&["emit-json", mode, source_path.to_str().unwrap()])
}
