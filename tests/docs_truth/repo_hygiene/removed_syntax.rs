use super::*;

#[test]
fn source_ast_no_longer_has_return_expression_nodes() {
    for path in [
        "src/ast/expressions.rs",
        "src/ast/typed.rs",
        "src/typechecker/expressions.rs",
        "src/typechecker/expressions/simple_forms.rs",
        "src/codegen/c/emit.rs",
        "src/codegen/c/types.rs",
    ] {
        let source = read(&path);
        for forbidden in [
            "Expression::Return",
            "TypedExprKind::Return",
            "check_return_expr",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} still contains dead return-expression support: {forbidden}"
            );
        }
    }
}

#[test]
fn promoted_stdlib_modules_do_not_use_removed_or_gated_syntax() {
    for path in [
        "stdlib/build.zen",
        "stdlib/collections/char.zen",
        "stdlib/collections/hashmap.zen",
        "stdlib/collections/linkedlist.zen",
        "stdlib/collections/queue.zen",
        "stdlib/collections/set.zen",
        "stdlib/collections/stack.zen",
        "stdlib/collections/string.zen",
        "stdlib/collections/vec.zen",
        "stdlib/compiler.zen",
        "stdlib/concurrency/actor/actor.zen",
        "stdlib/concurrency/actor/async_actor.zen",
        "stdlib/concurrency/actor/supervisor.zen",
        "stdlib/concurrency/actor/system.zen",
        "stdlib/concurrency/async/scheduler.zen",
        "stdlib/concurrency/async/task.zen",
        "stdlib/concurrency/primitives/atomic.zen",
        "stdlib/concurrency/primitives/futex.zen",
        "stdlib/concurrency/sync/barrier.zen",
        "stdlib/concurrency/sync/channel.zen",
        "stdlib/concurrency/sync/condvar.zen",
        "stdlib/concurrency/sync/mutex.zen",
        "stdlib/concurrency/sync/once.zen",
        "stdlib/concurrency/sync/rwlock.zen",
        "stdlib/concurrency/sync/semaphore.zen",
        "stdlib/concurrency/sync/thread.zen",
        "stdlib/concurrency/sync/waitgroup.zen",
        "stdlib/ffi.zen",
        "stdlib/fs.zen",
        "stdlib/io/eventfd.zen",
        "stdlib/io/files/copy.zen",
        "stdlib/io/files/dir.zen",
        "stdlib/io/files/file.zen",
        "stdlib/io/files/fs.zen",
        "stdlib/io/files/link.zen",
        "stdlib/io/files/splice.zen",
        "stdlib/io/files/stat.zen",
        "stdlib/io/inotify.zen",
        "stdlib/io/io.zen",
        "stdlib/io/mux/epoll.zen",
        "stdlib/io/mux/poll.zen",
        "stdlib/io/mux/uring.zen",
        "stdlib/io/net/socket.zen",
        "stdlib/io/net/unix_socket.zen",
        "stdlib/io/net/pipe.zen",
        "stdlib/io/signal.zen",
        "stdlib/io/terminal.zen",
        "stdlib/io/timerfd.zen",
        "stdlib/testing.zen",
        "stdlib/time.zen",
        "stdlib/math/math.zen",
        "stdlib/sys/env.zen",
        "stdlib/sys/memfd.zen",
        "stdlib/sys/uname.zen",
        "stdlib/sys/process/process.zen",
        "stdlib/sys/process/prctl.zen",
        "stdlib/sys/process/sched.zen",
        "stdlib/sys/random/getrandom.zen",
        "stdlib/sys/random/prng.zen",
        "stdlib/sys/resource.zen",
        "stdlib/sys/seccomp.zen",
        "stdlib/memory/allocator.zen",
        "stdlib/memory/arena.zen",
        "stdlib/memory/async_helpers.zen",
        "stdlib/memory/async_pool.zen",
        "stdlib/memory/heap.zen",
        "stdlib/memory/memory.zen",
        "stdlib/memory/mmap.zen",
        "stdlib/core/option.zen",
        "stdlib/core/result.zen",
        "stdlib/core/propagate.zen",
        "stdlib/core/buffer.zen",
        "stdlib/core/iterator.zen",
        "stdlib/core/ptr.zen",
        "stdlib/core/slice.zen",
    ] {
        let source = read(&path);
        assert!(
            !source.contains("return "),
            "{path} still uses the removed return keyword"
        );
        assert!(
            !source.contains(".raise("),
            "{path} still uses gated .raise() propagation"
        );
    }
}

#[test]
fn root_smoke_fixtures_do_not_use_removed_or_gated_syntax() {
    let mut paths = std::fs::read_dir(repo_root().join("tests"))
        .expect("read tests directory")
        .map(|entry| {
            let entry = entry.expect("tests directory entry should be readable");
            entry
                .path()
                .strip_prefix(repo_root())
                .expect("test path should be under repo root")
                .to_string_lossy()
                .into_owned()
        })
        .filter(|path| path.starts_with("tests/test_") && path.ends_with(".zen"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(!paths.is_empty(), "expected root smoke fixtures");

    for path in paths {
        let source = read(&path);
        assert!(
            !source.contains("return "),
            "{path} still uses the removed return keyword"
        );
        for gated_claim in [
            "@std.memory",
            "Heap.sync",
            "Arena.async",
            "Allocator",
            "ExecutionMode",
            "function coloring",
            "async/await",
        ] {
            assert!(
                !source.contains(gated_claim),
                "{path} still teaches gated allocator/effect syntax: {gated_claim}"
            );
        }
    }
}

#[test]
fn public_cast_fixture_uses_prefix_cast_syntax() {
    let source = read("tests/zen/cast.zen");
    assert!(
        !source
            .lines()
            .any(|line| line.trim_start().starts_with("y = x as ")
                || line.trim_start().starts_with("z = 3.14 as ")),
        "tests/zen/cast.zen should use prefix cast(value, Type), not infix as-cast syntax"
    );
    assert!(
        source.contains("cast("),
        "tests/zen/cast.zen should keep executable prefix cast coverage"
    );
}

#[test]
fn source_ast_does_not_carry_dead_char_literal_nodes() {
    for path in [
        "src/ast/expressions.rs",
        "src/typechecker/expressions.rs",
        "src/resolver/expression_validation.rs",
        "src/build_graph/lowering.rs",
        "src/typechecker/self_type_validation/expressions.rs",
        "src/typechecker/generic_type_reference_walker/expressions.rs",
        "src/typechecker/resolver_validation/local_traversal.rs",
        "src/typechecker/resolver_validation_support/expected_local_traversal.rs",
    ] {
        let source = read(path);
        for forbidden in ["CharLiteral", "TODO: implement char literal type"] {
            assert!(
                !source.contains(forbidden),
                "{path} still contains dead char-literal AST support: {forbidden}"
            );
        }
    }
}
