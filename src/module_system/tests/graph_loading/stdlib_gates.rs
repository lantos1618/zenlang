use super::*;

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
fn module_graph_gates_stdlib_compiler_facade_import_before_loading_sketch() {
    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "",
        "compiler.zen",
        "{ raw_allocate } = @std.compiler",
        "std compiler facade is gated",
        "compiler facade",
    );

    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "",
        "compiler.zen",
        "{ compiler } = @std",
        "std compiler facade is gated",
        "compiler facade",
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

#[test]
fn module_graph_gates_stdlib_io_uring_import_before_loading_sketch() {
    assert_graph_stdlib_import_is_gated_before_loading_sketch(
        "io/mux",
        "uring.zen",
        "{ IoUring } = @std.io.mux.uring",
        "std io_uring modules are gated",
        "io_uring",
    );
}
