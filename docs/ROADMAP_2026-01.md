# Zen Language Roadmap - January 2026

## Current Status: Late Alpha

The core compiler infrastructure is solid. Lexer, parser, type checker, and LLVM codegen work.
All major collection types now use safe `Ptr<T>` pointers with Zig-style allocator semantics.

---

## Memory Management Philosophy (Zig-Style)

Zen follows Zig's allocator pattern for memory management:

```zen
// Allocators are the memory managers - they own the memory
v = Vec<i32>.new(allocator)  // allocator provides memory
v.mut_ref().push(10)
v.mut_ref().free()           // allocator reclaims memory

// Types are "boxed" by allocators, not self-owned
// This enables:
// - Custom allocators (arena, pool, GPA, etc.)
// - Memory tracking and debugging
// - No hidden allocations
// - Explicit lifetime management
```

### Allocator Behavior
```zen
Allocator: behavior {
    allocate: (self: Self, size: usize) i64
    deallocate: (self: Self, ptr: i64, size: usize) void
    reallocate: (self: Self, ptr: i64, old_size: usize, new_size: usize) i64
}
```

All heap-allocated types (Vec, String, HashMap, Stack, Queue, Set) take an
`Allocator` and delegate memory operations to it. This is explicit, not magical.

---

## Priority 1: Complete Safe Pointer Migration

**Status: COMPLETE ✅**

| Component | Status | Notes |
|-----------|--------|-------|
| `Ptr<T>` definition | Done | `stdlib/core/ptr.zen` |
| `String` | Done | Uses `Ptr<u8>`, allocator-aware |
| `Vec<T>` | Done | Uses `Ptr<T>`, allocator-aware |
| `HashMap<K,V>` | Done | Uses `Vec<Entry<K,V>>` internally |
| `Stack<T>` | Done | Uses `Ptr<T>`, allocator-aware |
| `Queue<T>` | Done | Uses `Ptr<T>` (circular buffer) |
| `Set<T>` | Done | Wraps `HashMap<T, bool>` |

All collection types now use safe pointers with explicit allocator management.

---

## Priority 2: Standard Library Hardening

**Status: IN PROGRESS** (Linux x86-64)

### Core Types (Done)
- [x] `Option<T>` - fully integrated
- [x] `Result<T,E>` - fully integrated
- [x] `String` - growable, allocator-aware
- [x] `Vec<T>` - growable, allocator-aware
- [x] `HashMap<K,V>` - linear probing, FNV-1a hash

### Collections (Done)
- [x] `Stack<T>` - LIFO with push/pop/peek
- [x] `Queue<T>` - circular buffer with enqueue/dequeue
- [x] `Set<T>` - wraps HashMap<T, bool>
- [x] `LinkedList<T>` - doubly linked list with forward/reverse iterators

### I/O (Syscall Layer) - Linux x86-64
- [x] `File` - open, read, write, seek, mkdir, unlink (`io/file.zen`)
- [x] `Socket` - TCP/UDP, TcpListener, TcpStream, UdpSocket (`io/socket.zen`)
- [x] `Process` - fork, exec, wait, spawn, pipes, signals (`io/process.zen`)
- [x] `Dir` - stat, getdents64, readlink, directory iteration (`io/dir.zen`)
- [x] `Epoll` - epoll_create, epoll_ctl, epoll_wait, EventLoop (`io/epoll.zen`)
- [x] `Env` - getenv from /proc, read_args, getcwd (`io/env.zen`)
- [x] `Signal` - sigprocmask, sigaction, SignalFd, SigSet (`io/signal.zen`)
- [x] `Inotify` - inotify_init1, add_watch, FileWatcher (`io/inotify.zen`)
- [x] `Poll` - poll, ppoll, PollSet, PollEntry (`io/poll.zen`)
- [x] `Zerocopy` - sendfile, splice, tee, copy_file_range (`io/zerocopy.zen`)
- [x] `Statx` - extended file status with birth time (`io/statx.zen`)
- [x] `Ioctl` - device/terminal control (`io/ioctl.zen`)
- [x] `Pipe` - pipe, pipe2, PipeReader, PipeWriter (`io/pipe.zen`)
- [x] `Dup` - dup, dup2, dup3, stdio redirection (`io/dup.zen`)
- [ ] Darwin/Windows syscall support

### Sync (Syscall Layer) - Linux x86-64
- [x] `Futex` - futex_wait, futex_wake, Mutex, CondVar, Semaphore (`sync/futex.zen`)
- [x] `EventFd` - event notification fd, Notifier, Counter (`sync/eventfd.zen`)

### Time (Syscall Layer) - Linux x86-64
- [x] `Time` - clock_gettime, nanosleep, Instant, Stopwatch (`time/time.zen`)
- [x] `Timerfd` - timerfd_create, timerfd_settime, DeadlineTimer (`time/timerfd.zen`)

### Memory (Syscall Layer) - Linux x86-64
- [x] `Mmap` - mmap, mprotect, MemoryRegion, JitRegion (`memory/mmap.zen`)
- [x] `Memfd` - memfd_create, sealing, SharedBuffer (`memory/memfd.zen`)

### Random (Syscall Layer) - Linux x86-64
- [x] `Getrandom` - getrandom syscall, random_bytes, PRNG (`random/getrandom.zen`)

### Sys (Syscall Layer) - Linux x86-64
- [x] `Info` - uname, sysinfo, uptime, cpu_count (`sys/info.zen`)
- [x] `Prctl` - process control, rlimit, affinity (`sys/prctl.zen`)
- [x] `Pidfd` - race-free process management (`sys/pidfd.zen`)

### Net (Syscall Layer) - Linux x86-64
- [x] `Interface` - network interface info via ioctl (`net/interface.zen`)

### TODO: Syscall Modules to Add
- [ ] `io_uring` - High-performance async I/O
- [ ] `Fanotify` - Filesystem-wide notification
- [ ] `Xattr` - Extended file attributes
- [ ] `Mount` - Filesystem mounting, pivot_root
- [ ] `Clone3` - Modern process/thread creation
- [ ] `Seccomp` - BPF-based syscall filtering
- [ ] `Landlock` - Unprivileged filesystem sandboxing
- [ ] `Capability` - Linux capabilities management
- [ ] `Fcntl` - File control operations
- [ ] `Flock` - File locking

**Note:** All I/O uses direct syscalls, not libc wrappers:
```zen
// Direct syscall pattern
File.open = (path: String, flags: i32) Result<File, IoError>
File.read = (self: MutPtr<File>, buf: Ptr<u8>, len: usize) Result<usize, IoError>
```

### Known Limitation
Runtime tests for collections require the module import system to be complete.
Currently, complex module imports (e.g., `{ GPA } = @std.memory.gpa`) cause
"Unresolved generic type" errors during monomorphization. Rust tests verify compilation.

---

## Priority 3: Iterator System

**Status: PARTIAL ✅**

Design an iterator trait/behavior system:
```zen
Iter<T>: behavior {
    next: (self: MutPtr<Self>) Option<T>
}

// Enable: vec.iter().map(fn).filter(fn).collect()
```

### Tasks
- [x] Design `Range` iterator with `next()` method
- [x] Implement `VecIterator<T>` for `Vec<T>`
- [x] Add iterator combinators for Range: `sum`, `product`, `min`, `max`, `skip`, `take`
- [x] Add predicate methods: `any_ge`, `all_lt`, `find_ge`
- [x] Implement `HashMapIterator<K,V>` with `iter()`, `keys()`, `values()`
- [ ] Add `map`, `filter`, `fold`, `collect` combinators (needs closures)

**Note:** Full functional iterator chains (`.map(fn).filter(fn).collect()`) require
first-class closures which are not yet implemented. Current iterator methods are
specialized for common operations.

---

## Priority 4: Well-Known Types Refactor

**Status: Partial**

The compiler has hardcoded checks like `if name == "Option"`. Should use the
`WellKnownTypes` registry consistently.

### Files to Audit
- `src/codegen/llvm/expressions/enums.rs`
- `src/typechecker/mod.rs`
- `src/typechecker/inference.rs`

---

## Priority 5: FFI & Interop

**Status: Basic Working**

- [x] `load_library` / `get_symbol` - works
- [ ] `call_external` - stub in codegen
- [ ] `inline_c` - shells to clang (fragile)
- [ ] Struct layout compatibility with C

---

## Priority 6: Module System Improvements

**Status: Needs Work**

The module import system has issues with:
- Generic type resolution across module boundaries
- Monomorphization of imported generic types
- Type inference for behavior implementations

This blocks runtime testing of collections from .zen files.

---

## Priority 7: LSP Improvements

**Status: COMPLETE ✅**

### Full Feature Support
- [x] Hover provider with type info
- [x] Go-to-definition (including nested member access)
- [x] Type definition navigation
- [x] Find all references
- [x] Document highlight
- [x] Code completion with trigger characters (`.`, `:`, `@`, `?`)
- [x] Signature help for function calls
- [x] Document symbols
- [x] Workspace symbols
- [x] Code actions
- [x] Code lens
- [x] Document formatting
- [x] Rename with prepare provider
- [x] Folding ranges
- [x] Inlay hints
- [x] Call hierarchy (incoming/outgoing)
- [x] Semantic tokens (full + delta)
- [x] Incremental text sync

---

## Priority 8: Architecture Cleanup

**Status: IN PROGRESS**

### Completed ✅
- [x] Removed duplicate module declarations from `main.rs`
- [x] Deleted dead FFI module (`src/ffi/` - 1,455 LOC)
- [x] Deleted dead behaviors module (`src/behaviors/` - ~400 LOC)
- [x] Deleted `vec_support.rs` (326 LOC)
- [x] Deleted `stdlib_codegen/collections.rs` (670 LOC)
- [x] Integrated typechecker into main compilation pipeline

### Remaining
- [ ] Remove duplicate type inference from codegen (~1,000 LOC)
- [ ] Add type annotations to AST nodes (enable codegen to trust typechecker)
- [ ] Fix hardcoded generics in GenericTypeTracker
- [ ] Split giant modules (codegen/ 11.6K, lsp/ 12K)

---

## Stdlib File Count: 52 files

```
stdlib/
├── core/          (4 files) - iterator, option, propagate, ptr, result
├── collections/   (5 files) - hashmap, linkedlist, queue, set, stack
├── io/            (14 files) - dir, dup, env, epoll, file, inotify, io, ioctl, pipe, poll, process, signal, socket, statx, zerocopy
├── memory/        (4 files) - allocator, gpa, memfd, mmap
├── sync/          (2 files) - eventfd, futex
├── time/          (2 files) - time, timerfd
├── sys/           (3 files) - info, pidfd, prctl
├── net/           (2 files) - interface, net
├── random/        (1 file)  - getrandom
├── (root)         (9 files) - char, error, math, random, string, std, time, vec
├── build/         (1 file)  - build
├── compiler/      (1 file)  - compiler
├── ffi/           (1 file)  - ffi
├── fs/            (1 file)  - fs
└── testing/       (1 file)  - runner
```

---

## Testing Commands

```bash
# Build compiler
cargo build --release

# Run all Rust tests
cargo test

# Run allocator-specific tests
cargo test --test allocator_compilation

# Run demo project
./target/release/zen examples/demo_project/main.zen

# Run collections status check
./target/release/zen examples/test_collections.zen
```

---

## Architectural Goals

### Target Pipeline
```
Source
  ↓
Lexer (lexer.rs)
  ↓
Parser (parser/)
  ↓
═══════════════════════════
  SEMA (semantic analysis)
═══════════════════════════
  ├─ process_imports()
  ├─ execute_comptime()
  ├─ resolve_self_types()
  ├─ typecheck() ✅ NOW INTEGRATED
  └─ monomorphize()
═══════════════════════════
  ↓
Codegen (no type decisions!)
  ↓
LLVM IR
```

### Target Metrics
| Metric | Current | Target |
|--------|---------|--------|
| Total LOC | ~41,000 | < 35,000 |
| Dead code | ~0 | 0 |
| Max module LOC | 11,691 | < 2,000 |
| Typechecker integration | ✅ Done | Required |

---

## Self-Hosting Path

To achieve self-hosting, Zen needs:

1. **Intrinsics Only** - Rust compiler provides minimal intrinsics
2. **Stdlib in Zen** - All features built using intrinsics
3. **Parser in Zen** - Rewrite lexer/parser
4. **Typechecker in Zen** - Rewrite semantic analysis
5. **Codegen in Zen** - Either LLVM bindings or custom backend

Current intrinsics are a solid foundation:
- Memory: `raw_allocate`, `raw_deallocate`, `memcpy`, etc.
- Pointers: `gep`, `gep_struct`, `ptr_to_int`, etc.
- Types: `sizeof<T>`, `alignof<T>`
- Syscalls: `syscall0` - `syscall6`
- Atomics: `atomic_load`, `atomic_store`, `atomic_cas`, etc.
