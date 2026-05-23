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

#[derive(Clone, Copy)]
enum ModuleLoadPath {
    Imports,
    Graph,
}

fn assert_private_import_rejected(load_path: ModuleLoadPath) {
    let tmp = setup_temp_dir();

    write_private_add_module(&tmp);
    let main_path = write_main_importing_add(&tmp);

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();
    let result = match load_path {
        ModuleLoadPath::Imports => ms.load_with_imports(&main_path, &mut files).map(|_| ()),
        ModuleLoadPath::Graph => ms.load_module_graph(&main_path, &mut files).map(|_| ()),
    };

    assert!(
        result.is_err(),
        "private import should be rejected before module loading succeeds"
    );
    assert_error_contains(
        result,
        "not exported",
        "error should mention export visibility",
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

struct StdlibGateCase {
    sketch_dir: &'static str,
    sketch_file: &'static str,
    import_source: &'static str,
    expected_gate: &'static str,
    gate_name: &'static str,
}

const STDLIB_GATE_CASES: &[StdlibGateCase] = &[
    StdlibGateCase {
        sketch_dir: "concurrency/actor",
        sketch_file: "actor.zen",
        import_source: "{ Actor } = @std.concurrency.actor.actor",
        expected_gate: "std actor framework modules are gated",
        gate_name: "actor",
    },
    StdlibGateCase {
        sketch_dir: "memory",
        sketch_file: "allocator.zen",
        import_source: "{ Allocator } = @std.memory.allocator",
        expected_gate: "std allocator modules are gated",
        gate_name: "allocator",
    },
    StdlibGateCase {
        sketch_dir: "",
        sketch_file: "compiler.zen",
        import_source: "{ raw_allocate } = @std.compiler",
        expected_gate: "std compiler facade is gated",
        gate_name: "compiler facade",
    },
    StdlibGateCase {
        sketch_dir: "",
        sketch_file: "compiler.zen",
        import_source: "{ compiler } = @std",
        expected_gate: "std compiler facade is gated",
        gate_name: "compiler facade root",
    },
    StdlibGateCase {
        sketch_dir: "concurrency/async",
        sketch_file: "scheduler.zen",
        import_source: "{ Scheduler } = @std.concurrency.async.scheduler",
        expected_gate: "std async runtime modules are gated",
        gate_name: "async",
    },
    StdlibGateCase {
        sketch_dir: "concurrency/sync",
        sketch_file: "channel.zen",
        import_source: "{ Channel } = @std.concurrency.sync.channel",
        expected_gate: "std sync runtime modules are gated",
        gate_name: "sync",
    },
    StdlibGateCase {
        sketch_dir: "io/mux",
        sketch_file: "uring.zen",
        import_source: "{ IoUring } = @std.io.mux.uring",
        expected_gate: "std io_uring modules are gated",
        gate_name: "io_uring",
    },
];

fn assert_stdlib_gate_cases_are_gated_before_loading_sketch(load_path: StdlibGateLoadPath) {
    for case in STDLIB_GATE_CASES {
        assert_stdlib_import_is_gated_before_loading_sketch_on_path(load_path, case);
    }
}

fn assert_stdlib_import_is_gated_before_loading_sketch_on_path(
    load_path: StdlibGateLoadPath,
    case: &StdlibGateCase,
) {
    let tmp = setup_temp_dir();
    let sketch_dir = tmp.path().join("stdlib").join(case.sketch_dir);
    fs::create_dir_all(&sketch_dir).unwrap();
    fs::write(sketch_dir.join(case.sketch_file), "this is not valid zen\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        format!("{}\n\nmain = () i32 {{ 0 }}\n", case.import_source),
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
        messages.contains(case.expected_gate),
        "expected {} stdlib gate diagnostic, got {messages}",
        case.gate_name
    );
    assert!(
        !messages.contains("expected") && !messages.contains("unexpected token"),
        "{} stdlib gate should not leak parser diagnostics from sketches, got {messages}",
        case.gate_name
    );
}
