# Allocator Trait Redesign - Unified Concurrency Model

## The Core Insight

**Anything async requires memory.** Buffers, state, callbacks - all need allocation.
Since allocators already flow through every function (Zig-style), the allocator
is the natural place to carry execution context.

This avoids the **red/blue function coloring problem** entirely:
- Same function works sync OR async
- Behavior determined by what allocator you pass
- No `async`/`await` keywords needed
- Similar to Nim's `multisync` but built into the allocator

## The Vision

```zen
// ONE function - no coloring, no async keyword
fetch = (url: String, alloc: Allocator) Result<Data, Error> {
    client = HttpClient(alloc)
    @this.defer(client.deinit())
    response = client.get(url)
    return response
}

main = () void {
    gpa = GPA.new()
    async_pool = AsyncPool.init().unwrap()

    // Same function, sync execution (blocks)
    data1 = fetch("https://api.example.com", gpa)

    // Same function, async execution (non-blocking)
    data2 = fetch("https://api.example.com", async_pool)
}
```

## Why This Works

1. **HttpClient uses allocator for buffers** - needs memory anyway
2. **HttpClient calls `alloc.schedule_read()`** - allocator handles I/O
3. **For GPA**: `schedule_read` does blocking syscall, returns immediately
4. **For AsyncPool**: `schedule_read` submits to io_uring, returns op_id
5. **Both call `alloc.wait()`** - GPA no-ops, AsyncPool blocks until complete

The function doesn't know or care if it's sync or async. The allocator decides.

## Previous Design (REJECTED)

I initially proposed separating `Allocator` (memory) and `ExecutionContext` (I/O):

```zen
// BAD - reintroduces function coloring!
fetch = (url: String, alloc: Allocator, ctx: ExecutionContext) Result<Data, Error>
```

This is wrong because:
- Functions need TWO parameters for async capability
- Functions that take only `Allocator` can't do async I/O
- We've recreated the red/blue problem with different names

## Correct Design: Unified Allocator

Every allocator implements the FULL interface - memory AND I/O:

```zen
Allocator: {
    // === Memory Operations ===
    allocate: (self, size: usize) RawPtr<u8>,
    deallocate: (self, ptr: RawPtr<u8>, size: usize) void,
    reallocate: (self, ptr: RawPtr<u8>, old_size: usize, new_size: usize) RawPtr<u8>,

    // === Execution Context ===
    mode: (self) ExecutionMode,
    schedule_read: (self, fd: i32, buf: RawPtr<u8>, len: usize, offset: i64, callback: CompletionFn, user_data: u64) i64,
    schedule_write: (self, fd: i32, buf: RawPtr<u8>, len: usize, offset: i64, callback: CompletionFn, user_data: u64) i64,
    schedule_accept: (self, listen_fd: i32, addr: RawPtr<u8>, addrlen: RawPtr<u8>, callback: CompletionFn, user_data: u64) i64,
    schedule_connect: (self, fd: i32, addr: RawPtr<u8>, addrlen: i32, callback: CompletionFn, user_data: u64) i64,
    poll: (self) i32,
    wait: (self) i32,
    cancel: (self, op_id: u64) bool
}
```

### GPA (Sync Allocator)

```zen
GPA.implements(Allocator, {
    // Memory - use system malloc
    allocate = (self: GPA, size: usize) RawPtr<u8> {
        return compiler.raw_allocate(size)
    },
    deallocate = (self: GPA, ptr: RawPtr<u8>, size: usize) void {
        compiler.raw_deallocate(ptr, size)
    },
    reallocate = (self: GPA, ptr: RawPtr<u8>, old_size: usize, new_size: usize) RawPtr<u8> {
        return compiler.raw_reallocate(ptr, old_size, new_size)
    },

    // Execution - BLOCKING I/O
    mode = (self: GPA) ExecutionMode { return ExecutionMode.Sync },

    schedule_read = (self: GPA, fd: i32, buf: RawPtr<u8>, len: usize, offset: i64, callback: CompletionFn, user_data: u64) i64 {
        // Do blocking read RIGHT NOW
        buf_i64 = compiler.ptr_to_int(buf)
        result = compiler.syscall4(0, fd, buf_i64, len, offset)  // SYS_READ/PREAD
        // Call callback immediately
        callback(user_data, result)
        return 0  // No pending operation
    },

    schedule_write = (self: GPA, fd: i32, buf: RawPtr<u8>, len: usize, offset: i64, callback: CompletionFn, user_data: u64) i64 {
        buf_i64 = compiler.ptr_to_int(buf)
        result = compiler.syscall4(1, fd, buf_i64, len, offset)  // SYS_WRITE/PWRITE
        callback(user_data, result)
        return 0
    },

    // ... similar for accept, connect

    poll = (self: GPA) i32 { return 0 },     // Nothing to poll - already done
    wait = (self: GPA) i32 { return 0 },     // Nothing to wait for
    cancel = (self: GPA, op_id: u64) bool { return false }  // Can't cancel completed ops
})
```

### AsyncPool (Async Allocator)

```zen
AsyncPool.implements(Allocator, {
    // Memory - bump allocator from mmap'd pool
    allocate = (self: AsyncPool, size: usize) RawPtr<u8> { ... },
    deallocate = (self: AsyncPool, ptr: RawPtr<u8>, size: usize) void { ... },
    reallocate = (self: AsyncPool, ptr: RawPtr<u8>, old_size: usize, new_size: usize) RawPtr<u8> { ... },

    // Execution - NON-BLOCKING via io_uring
    mode = (self: AsyncPool) ExecutionMode { return ExecutionMode.Async },

    schedule_read = (self: AsyncPool, fd: i32, buf: RawPtr<u8>, len: usize, offset: i64, callback: CompletionFn, user_data: u64) i64 {
        // Submit to io_uring - returns immediately
        op_id = self.alloc_pending_op(callback, user_data)
        self.ring.prep_read(fd, buf, len, offset, op_id)
        self.ring.submit()
        return op_id  // Caller can track/cancel this
    },

    poll = (self: AsyncPool) i32 {
        // Check io_uring for completions, invoke callbacks
        // Returns number of completed operations
    },

    wait = (self: AsyncPool) i32 {
        // Block until at least one completion
        // Then invoke callbacks
    },

    cancel = (self: AsyncPool, op_id: u64) bool {
        // Cancel pending io_uring operation
    }
})
```

## How Code Uses This

### Low-level I/O code

```zen
read_file = (fd: i32, buf: RawPtr<u8>, len: usize, alloc: Allocator) i64 {
    result ::= 0

    on_complete = (user_data: u64, res: i64) void {
        result = res
    }

    alloc.schedule_read(fd, buf, len, 0, on_complete, 0)
    alloc.wait()  // Sync: no-op. Async: blocks until done.

    return result
}
```

### High-level code (doesn't care about sync/async)

```zen
fetch = (url: String, alloc: Allocator) Result<String, Error> {
    // HttpClient internally uses alloc.schedule_read, alloc.wait, etc.
    client = HttpClient(alloc)
    return client.get(url)
}

// Both work with the SAME function:
data = fetch(url, gpa)          // Sync
data = fetch(url, async_pool)   // Async
```

## Integration with Actors

Actors already use allocators for mailboxes. With unified allocator:

```zen
Actor<M>: {
    mailbox: Mailbox<M>,
    alloc: Allocator
}

Actor.receive = (self: Actor<M>) Option<M> {
    // Uses self.alloc for I/O internally
    // Sync actor blocks, async actor yields
}
```

An actor with `GPA` is a blocking actor.
An actor with `AsyncPool` is a non-blocking actor that integrates with io_uring.

Same actor code, different execution model based on allocator.

## Implementation Plan

### File Structure

```
stdlib/memory/
├── allocator.zen       # Allocator trait + ExecutionMode + CompletionFn
├── heap.zen            # Heap namespace + HeapSync allocator
├── arena.zen           # Arena namespace + ArenaAsync allocator
├── async_helpers.zen   # AsyncOp, pending op tracking (shared by async allocators)
└── mod.zen             # Re-exports for convenience
```

### Phase 1: allocator.zen (DONE)
- [x] Unified Allocator trait with memory + I/O methods

### Phase 2: heap.zen (NEW)
- [ ] Create `HeapSync` struct
- [ ] Implement `Allocator` trait with blocking I/O
- [ ] Create `Heap` namespace with `.sync()` factory
- [ ] Delete old `gpa.zen`

```zen
// Usage:
{ Heap } = @std.memory.heap
alloc = Heap.sync()
ptr = alloc.allocate(64)
alloc.schedule_read(fd, ptr, 64, 0, callback, 0)  // Blocks!
alloc.deallocate(ptr, 64)
```

### Phase 3: arena.zen (REPLACES async_pool.zen)
- [ ] Create `ArenaAsync` struct (bump allocator + io_uring)
- [ ] Implement `Allocator` trait with non-blocking I/O
- [ ] Create `Arena` namespace with `.async()` factory
- [ ] Keep `.sync()` for future ArenaSync
- [ ] Delete old `async_pool.zen`

```zen
// Usage:
{ Arena } = @std.memory.arena
alloc = Arena.async(4 * 1024 * 1024)  // 4MB pool
ptr = alloc.allocate(64)
alloc.schedule_read(fd, ptr, 64, 0, callback, 0)  // Non-blocking!
alloc.wait()  // Block until complete
alloc.deallocate(ptr, 64)
```

### Phase 4: async_helpers.zen
- [ ] Move `AsyncOp`, `PendingOp` structs here
- [ ] Shared by all async allocators (ArenaAsync, future HeapAsync, etc.)

### Phase 5: Cleanup
- [ ] Delete `gpa.zen` (replaced by heap.zen)
- [ ] Delete `async_pool.zen` (replaced by arena.zen)
- [ ] Delete `async_allocator.zen` (split into allocator.zen + async_helpers.zen)
- [ ] Update any imports in stdlib that used old names

### Phase 6: Test
- [ ] Write test using SAME function with `Heap.sync()` and `Arena.async()`
- [ ] Verify behavior matches expectations

## API Examples

```zen
{ Heap } = @std.memory.heap
{ Arena } = @std.memory.arena

// The SAME function works with any allocator
read_file = (path: String, alloc: Allocator) Result<String, Error> {
    fd = open(path)
    buf = alloc.allocate(4096)

    bytes_read ::= 0
    on_complete = (user_data: u64, result: i64) void {
        bytes_read = result
    }

    alloc.schedule_read(fd, buf, 4096, 0, on_complete, 0)
    alloc.wait()

    // ... convert buf to string ...
    alloc.deallocate(buf, 4096)
    return Ok(result)
}

main = () i32 {
    // Sync - blocks during schedule_read
    sync_alloc = Heap.sync()
    data1 = read_file("test.txt", sync_alloc)

    // Async - schedule_read returns immediately, wait() blocks
    async_alloc = Arena.async(4 * 1024 * 1024)
    data2 = read_file("test.txt", async_alloc)

    return 0
}
```

## Allocator Naming & Types

### Two Orthogonal Concerns

**Memory Strategy** (HOW memory is managed):
| Strategy | Description | Use Case |
|----------|-------------|----------|
| Heap | malloc/free, general purpose | Default, most code |
| Arena | Sequential alloc, bulk free | Request handling, parsing |
| Pool | Fixed-size blocks | Game objects, connections |
| Stack | LIFO allocation | Temporary scratch space |
| Page | Direct mmap | Large allocations |

**Execution Mode** (HOW I/O is handled):
| Mode | Description | Implementation |
|------|-------------|----------------|
| Sync | Blocking syscalls | read(), write(), etc. |
| Async | Non-blocking | io_uring on Linux |

### Naming Options

**Option A: Compound names**
```zen
HeapSync, HeapAsync, ArenaSync, ArenaAsync, PoolSync, PoolAsync
```

**Option B: Factory methods**
```zen
Heap.sync()           // → HeapSync allocator
Heap.async()          // → HeapAsync allocator
Arena.sync(4096)      // → ArenaSync allocator
Arena.async(4096)     // → ArenaAsync allocator
```

**Option C: Generics (future)**
```zen
Sync<Heap>, Async<Heap>, Sync<Arena>, Async<Arena>
```

### Proposed Type Hierarchy

```
                    Allocator (trait)
                         │
        ┌────────────────┼────────────────┐
        │                │                │
    HeapSync        ArenaSync        PoolSync
    HeapAsync       ArenaAsync       PoolAsync
```

All implement the same `Allocator` trait, so any function taking `Allocator` works with any of them.

### Default / Convenience

```zen
// Full explicit
alloc = HeapSync.new()

// Or factory
alloc = Heap.sync()

// Or maybe just
alloc = Allocator.default()  // Returns HeapSync
```

### Current Names → New Names

| Current | New | Notes |
|---------|-----|-------|
| GPA | HeapSync | General purpose, blocking |
| AsyncPool | ArenaAsync | Bump allocator + io_uring |

---

## Open Questions

1. **Should we keep a "MemoryAllocator" trait for simple cases?**
   - Some code only needs memory, not I/O
   - Could have `MemoryAllocator` (just memory) and `Allocator` (memory + I/O)
   - But this might reintroduce complexity...
   - **Decision**: No. Keep it simple. One trait. GPA's I/O methods are trivial.

2. **What about allocators that CAN'T do I/O?**
   - Arena allocators, stack allocators, etc.
   - They can implement I/O methods as "not supported" (return error)
   - Or delegate to a parent allocator
   - **Decision**: They implement blocking I/O like GPA (simple syscalls)

3. **Thread safety?**
   - GPA is thread-safe (malloc is thread-safe)
   - AsyncPool is NOT thread-safe (io_uring is per-thread)
   - Actors handle this naturally (message passing)
   - **Decision**: Document thread safety per allocator type
