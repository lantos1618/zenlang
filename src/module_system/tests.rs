use super::*;
use std::fs;
use std::path::PathBuf;

mod cache_and_ids;
mod graph_loading;
mod imports;
mod stdlib_gates;
mod visibility;

fn setup_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn write_zen_file(path: PathBuf, source: &str) -> PathBuf {
    fs::write(&path, source).unwrap();
    path
}

fn write_module(tmp: &tempfile::TempDir, name: &str, source: &str) -> PathBuf {
    write_zen_file(tmp.path().join(format!("{name}.zen")), source)
}

fn write_main(tmp: &tempfile::TempDir, source: &str) -> PathBuf {
    write_zen_file(tmp.path().join("main.zen"), source)
}

fn write_public_add_module(tmp: &tempfile::TempDir) -> PathBuf {
    write_module(tmp, "math", "pub add = (a: i32, b: i32) i32 { a + b }\n")
}

fn write_private_add_module(tmp: &tempfile::TempDir) -> PathBuf {
    write_module(tmp, "math", "add = (a: i32, b: i32) i32 { a + b }\n")
}

fn write_main_importing_add(tmp: &tempfile::TempDir) -> PathBuf {
    write_main(tmp, "{ add } = math\n\nmain = () i32 { add(1, 2) }\n")
}

fn first_error_message<T>(result: Result<T, Vec<CompileError>>) -> String {
    match result {
        Ok(_) => panic!("expected module-system error"),
        Err(errors) => format!("{}", errors[0]),
    }
}

fn assert_error_contains<T>(result: Result<T, Vec<CompileError>>, expected: &str, context: &str) {
    let msg = first_error_message(result);
    assert!(
        msg.contains(expected),
        "{context}; expected `{expected}`, got: {msg}"
    );
}

fn module_function_names(program: &Program) -> Vec<&str> {
    program
        .declarations
        .iter()
        .filter_map(|declaration| declaration.name())
        .collect()
}

#[derive(Clone, Copy)]
enum StdlibGateLoadPath {
    Imports,
    Graph,
}

fn assert_stdlib_import_is_gated_before_loading_sketch(
    sketch_dir: &str,
    sketch_file: &str,
    import_source: &str,
    expected_gate: &str,
    gate_name: &str,
) {
    assert_stdlib_import_is_gated_before_loading_sketch_on_path(
        StdlibGateLoadPath::Imports,
        sketch_dir,
        sketch_file,
        import_source,
        expected_gate,
        gate_name,
    );
}

fn assert_graph_stdlib_import_is_gated_before_loading_sketch(
    sketch_dir: &str,
    sketch_file: &str,
    import_source: &str,
    expected_gate: &str,
    gate_name: &str,
) {
    assert_stdlib_import_is_gated_before_loading_sketch_on_path(
        StdlibGateLoadPath::Graph,
        sketch_dir,
        sketch_file,
        import_source,
        expected_gate,
        gate_name,
    );
}

fn assert_stdlib_import_is_gated_before_loading_sketch_on_path(
    load_path: StdlibGateLoadPath,
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

    let result = match load_path {
        StdlibGateLoadPath::Imports => ms.load_with_imports(&main_path, &mut files).map(|_| ()),
        StdlibGateLoadPath::Graph => ms.load_module_graph(&main_path, &mut files).map(|_| ()),
    };
    let errors = result.expect_err("stdlib import should be gated before parsing sketches");
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
