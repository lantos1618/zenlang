use super::*;

#[test]
fn std_imports_are_skipped() {
    let tmp = setup_temp_dir();

    let main_path = tmp.path().join("main.zen");
    fs::write(&main_path, "{ io } = std\n\nmain = () i32 { 0 }\n").unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::new();

    let program = ms.load_with_imports(&main_path, &mut files).unwrap();
    assert!(program
        .declarations
        .iter()
        .any(|d| d.name() == Some("main")));
}

#[test]
fn stdlib_submodule_import_loads_through_module_system() {
    let tmp = setup_temp_dir();
    let stdlib = tmp.path().join("stdlib");
    fs::create_dir(&stdlib).unwrap();
    fs::write(&stdlib.join("math.zen"), "pub answer = () i32 { 42 }\n").unwrap();

    let main_path = tmp.path().join("main.zen");
    fs::write(
        &main_path,
        "{ answer } = std.math\n\nmain = () i32 { answer() }\n",
    )
    .unwrap();

    let mut files = FileTable::new();
    let mut ms = ModuleSystem::with_stdlib_root(stdlib.clone());

    let program = ms.load_with_imports(&main_path, &mut files).unwrap();
    let func_names = module_function_names(&program);

    assert!(func_names.contains(&"main"));
    assert!(func_names.contains(&"answer"));
    assert_eq!(
        files.file_count(),
        2,
        "main and stdlib module should both be loaded"
    );

    let std_key = stdlib
        .join("math.zen")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let std_info = ms.module_info(&std_key).expect("stdlib module info");
    assert_eq!(std_info.package_id.0, 1, "stdlib modules use package 1");
}

#[test]
fn stdlib_actor_framework_import_is_gated_before_loading_sketch() {
    assert_stdlib_import_is_gated_before_loading_sketch(
        "concurrency/actor",
        "actor.zen",
        "{ Actor } = @std.concurrency.actor.actor",
        "std actor framework modules are gated",
        "actor",
    );
}

#[test]
fn stdlib_allocator_import_is_gated_before_loading_sketch() {
    assert_stdlib_import_is_gated_before_loading_sketch(
        "memory",
        "allocator.zen",
        "{ Allocator } = @std.memory.allocator",
        "std allocator modules are gated",
        "allocator",
    );
}

#[test]
fn stdlib_async_runtime_import_is_gated_before_loading_sketch() {
    assert_stdlib_import_is_gated_before_loading_sketch(
        "concurrency/async",
        "scheduler.zen",
        "{ Scheduler } = @std.concurrency.async.scheduler",
        "std async runtime modules are gated",
        "async",
    );
}

#[test]
fn stdlib_sync_runtime_import_is_gated_before_loading_sketch() {
    assert_stdlib_import_is_gated_before_loading_sketch(
        "concurrency/sync",
        "channel.zen",
        "{ Channel } = @std.concurrency.sync.channel",
        "std sync runtime modules are gated",
        "sync",
    );
}

#[test]
fn stdlib_io_uring_import_is_gated_before_loading_sketch() {
    assert_stdlib_import_is_gated_before_loading_sketch(
        "io/mux",
        "uring.zen",
        "{ IoUring } = @std.io.mux.uring",
        "std io_uring modules are gated",
        "io_uring",
    );
}
