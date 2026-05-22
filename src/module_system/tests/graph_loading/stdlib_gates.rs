use super::*;

fn assert_graph_stdlib_import_is_gated_before_loading_sketch(
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
        .load_module_graph(&main_path, &mut files)
        .expect_err("stdlib import should be gated before graph loading sketches");
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

#[test]
fn module_graph_gates_stdlib_actor_framework_import_before_loading_sketch() {
    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "concurrency/actor",
        "actor.zen",
        "{ Actor } = @std.concurrency.actor.actor",
        "std actor framework modules are gated",
        "actor",
    );
}

#[test]
fn module_graph_gates_stdlib_allocator_import_before_loading_sketch() {
    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "memory",
        "allocator.zen",
        "{ Allocator } = @std.memory.allocator",
        "std allocator modules are gated",
        "allocator",
    );
}

#[test]
fn module_graph_gates_stdlib_async_runtime_import_before_loading_sketch() {
    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "concurrency/async",
        "scheduler.zen",
        "{ Scheduler } = @std.concurrency.async.scheduler",
        "std async runtime modules are gated",
        "async",
    );
}

#[test]
fn module_graph_gates_stdlib_sync_runtime_import_before_loading_sketch() {
    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "concurrency/sync",
        "channel.zen",
        "{ Channel } = @std.concurrency.sync.channel",
        "std sync runtime modules are gated",
        "sync",
    );
}
