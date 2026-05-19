use super::*;
use std::fs;

mod cache_and_ids;
mod graph_loading;
mod imports;
mod stdlib_gates;
mod visibility;

fn setup_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn module_function_names(program: &Program) -> Vec<&str> {
    program
        .declarations
        .iter()
        .filter_map(|declaration| declaration.name())
        .collect()
}

fn assert_stdlib_import_is_gated_before_loading_sketch(
    sketch_dir: &str,
    sketch_file: &str,
    import_source: &str,
    expected_gate: &str,
    gate_name: &str,
) {
    let tmp = setup_temp_dir();
    let sketch_dir = tmp.path().join("stdlib").join(sketch_dir);
    fs::create_dir_all(&sketch_dir).unwrap();
    fs::write(sketch_dir.join(sketch_file), "this is not valid zen\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        format!("{import_source}\n\nmain = () i32 {{ 0 }}\n"),
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::with_stdlib_root(tmp.path().join("stdlib"));

    let errors = ms
        .load_with_imports(&main_path, &mut files)
        .expect_err("stdlib import should be gated before parsing sketches");
    let messages = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        messages.contains(expected_gate),
        "expected {gate_name} stdlib gate diagnostic, got {messages}"
    );
    assert!(
        !messages.contains("expected") && !messages.contains("unexpected token"),
        "{gate_name} stdlib gate should not leak parser diagnostics from sketches, got {messages}"
    );
}
