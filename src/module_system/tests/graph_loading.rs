use super::*;

#[test]
fn module_graph_records_imports_without_merging_declarations() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 {\n    add(1, 2)\n}\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let graph = ms.load_module_graph(&main_path, &mut files).unwrap();
    let entry = graph.module(graph.entry).expect("entry module");
    let entry_names: Vec<&str> = entry
        .program
        .declarations
        .iter()
        .filter_map(|d| d.name())
        .collect();

    assert!(entry_names.contains(&"main"));
    assert!(
        !entry_names.contains(&"add"),
        "module graph must not merge imported declarations into the entry AST"
    );
    assert_eq!(entry.imports.len(), 1);

    let binding = &entry.imports[0];
    assert_eq!(binding.local_name, "add");
    assert_eq!(binding.source_symbol, "add");

    let math_key = math_path.canonicalize().unwrap().display().to_string();
    let math_module = graph
        .module_by_path(&math_key)
        .expect("imported module by canonical path");
    assert_eq!(binding.source_module, math_module.info.id);
    assert!(math_module
        .program
        .declarations
        .iter()
        .any(|d| d.name() == Some("add")));
}

#[test]
fn module_graph_records_resolver_symbols_per_module() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(
        &math_path,
        "pub Point: { x: i32 }\npub add = (a: i32, b: i32) i32 { a + b }\n",
    )
    .unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ add, Point } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let graph = ms.load_module_graph(&main_path, &mut files).unwrap();
    let entry = graph.module(graph.entry).expect("entry module");
    assert!(entry
        .symbols
        .lookup(crate::resolver::Namespace::Value, "main")
        .is_some());
    assert!(entry
        .symbols
        .lookup(crate::resolver::Namespace::Import, "add")
        .is_some());
    assert!(entry
        .symbols
        .lookup(crate::resolver::Namespace::Import, "Point")
        .is_some());

    let math_key = math_path.canonicalize().unwrap().display().to_string();
    let math_module = graph
        .module_by_path(&math_key)
        .expect("imported module by canonical path");
    assert!(math_module
        .symbols
        .lookup(crate::resolver::Namespace::Value, "add")
        .is_some());
    assert!(math_module
        .symbols
        .lookup(crate::resolver::Namespace::Type, "Point")
        .is_some());
}

#[test]
fn module_graph_rejects_resolver_errors_in_loaded_modules() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "pub add = () Missing { 0 }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(&main_path, "{ add } = math\n\nmain = () i32 { add() }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_module_graph(&main_path, &mut files);
    assert!(
        result.is_err(),
        "module graph should reject resolver diagnostics from dependency modules"
    );
    let msg = format!("{}", result.unwrap_err()[0]);
    assert!(
        msg.contains("unknown type symbol 'Missing'"),
        "error should surface resolver diagnostic, got: {msg}"
    );
}

#[test]
fn module_graph_reuses_export_visibility_errors() {
    let tmp = setup_temp_dir();

    let math_path = tmp.path().join("math.zen");
    fs::write(&math_path, "add = (a: i32, b: i32) i32 { a + b }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_module_graph(&main_path, &mut files);
    assert!(result.is_err(), "private graph import should be rejected");
    let msg = format!("{}", result.unwrap_err()[0]);
    assert!(
        msg.contains("not exported"),
        "error should mention export visibility, got: {msg}"
    );
}

#[test]
fn module_graph_gates_stdlib_actor_framework_import_before_loading_sketch() {
    let tmp = setup_temp_dir();
    let actor_dir = tmp.path().join("stdlib/concurrency/actor");
    fs::create_dir_all(&actor_dir).unwrap();
    fs::write(actor_dir.join("actor.zen"), "this is not valid zen\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ Actor } = @std.concurrency.actor.actor\n\nmain = () i32 { 0 }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::with_stdlib_root(tmp.path().join("stdlib"));

    let errors = ms
        .load_module_graph(&main_path, &mut files)
        .expect_err("actor stdlib imports should be gated before graph loading sketches");
    let messages = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        messages.contains("std actor framework modules are gated"),
        "expected actor stdlib gate diagnostic, got {messages}"
    );
    assert!(
        !messages.contains("expected") && !messages.contains("unexpected token"),
        "actor stdlib gate should not leak parser diagnostics from sketches, got {messages}"
    );
}

#[test]
fn module_graph_gates_stdlib_allocator_import_before_loading_sketch() {
    let tmp = setup_temp_dir();
    let memory_dir = tmp.path().join("stdlib/memory");
    fs::create_dir_all(&memory_dir).unwrap();
    fs::write(memory_dir.join("allocator.zen"), "this is not valid zen\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ Allocator } = @std.memory.allocator\n\nmain = () i32 { 0 }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::with_stdlib_root(tmp.path().join("stdlib"));

    let errors = ms
        .load_module_graph(&main_path, &mut files)
        .expect_err("allocator stdlib imports should be gated before graph loading sketches");
    let messages = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        messages.contains("std allocator modules are gated"),
        "expected allocator stdlib gate diagnostic, got {messages}"
    );
    assert!(
        !messages.contains("expected") && !messages.contains("unexpected token"),
        "allocator stdlib gate should not leak parser diagnostics from sketches, got {messages}"
    );
}

#[test]
fn module_graph_gates_stdlib_async_runtime_import_before_loading_sketch() {
    let tmp = setup_temp_dir();
    let async_dir = tmp.path().join("stdlib/concurrency/async");
    fs::create_dir_all(&async_dir).unwrap();
    fs::write(async_dir.join("scheduler.zen"), "this is not valid zen\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ Scheduler } = @std.concurrency.async.scheduler\n\nmain = () i32 { 0 }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::with_stdlib_root(tmp.path().join("stdlib"));

    let errors = ms
        .load_module_graph(&main_path, &mut files)
        .expect_err("async stdlib imports should be gated before graph loading sketches");
    let messages = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        messages.contains("std async runtime modules are gated"),
        "expected async stdlib gate diagnostic, got {messages}"
    );
    assert!(
        !messages.contains("expected") && !messages.contains("unexpected token"),
        "async stdlib gate should not leak parser diagnostics from sketches, got {messages}"
    );
}

#[test]
fn module_graph_detects_circular_imports() {
    let tmp = setup_temp_dir();

    let a_path = tmp.path().join("a.zen");
    fs::write(&a_path, "{ bar } = b\n\npub foo = () i32 { 1 }\n").unwrap();

    let b_path = tmp.path().join("b.zen");
    fs::write(&b_path, "{ foo } = a\n\npub bar = () i32 { 2 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let result = ms.load_module_graph(&a_path, &mut files);
    assert!(result.is_err(), "circular graph import should be rejected");
    let msg = format!("{}", result.unwrap_err()[0]);
    assert!(
        msg.contains("circular import"),
        "error should mention circular import, got: {msg}"
    );
}
