# Zen Compiler Rewrite — C Backend

## Why Rewrite

The current compiler targets LLVM IR. Everything LLVM does for us, C already provides:

| LLVM gives us             | C equivalent                                    |
|---------------------------|-------------------------------------------------|
| Optimizations             | Compile with `clang -O3` (clang IS LLVM)        |
| Platform support          | C runs on more targets than LLVM IR             |
| Debug info (DWARF)        | `cc -g` — automatic                             |
| ABI / FFI                 | C *is* the ABI                                  |
| Inline assembly           | `__asm__` / platform asm blocks                 |
| Atomics                   | C11 `_Atomic`, `<stdatomic.h>`                  |
| Register allocation       | Free — the C compiler handles it                |
| Calling conventions       | `__attribute__((cdecl))` etc.                   |

What LLVM gives us that C doesn't:

| Feature                   | Needed today? |
|---------------------------|---------------|
| Custom optimization passes| No            |
| JIT compilation           | No            |
| No C dependency           | Already linking libc anyway |

**Conclusion**: LLVM is the hardest part of the compiler solving problems we don't have.
A C backend gets us running programs immediately with simpler code.

### Honest Tradeoffs

The C backend isn't free. What we give up:

| Tradeoff                    | Impact   | Mitigation                                      |
|-----------------------------|----------|-------------------------------------------------|
| Two-stage compile (zen→C→binary) | Slower builds | C compilation is fast; LLVM was the slow part anyway |
| Less control over generated code | Can't hand-tune IR | `clang -O3` handles 99% of what we'd do manually |
| Debug steps through C, not Zen | Bad DX without work | `#line` directives map C back to .zen source    |
| C type system limits codegen | Some patterns are awkward | Tagged unions, void* casts — ugly but correct   |
| Dependency on a C compiler  | Extra toolchain requirement | Every system already has `cc`                   |
| Generated C is unreadable   | Harder to debug codegen bugs | Emit formatted C with comments; snapshot tests  |

These are real costs. We accept them because the LLVM costs are worse:
LLVM system dep (2GB+), inkwell API churn, 30s+ build times, LLVM version lock-in,
and all of it solving optimization problems we haven't needed yet.

### What the rewrite deletes

- `src/codegen/llvm/` — all LLVM IR generation (~12K lines, 2K of which is shadow type system)
- `src/typechecker/` — rewrite to produce Typed AST (~7K lines; pipeline shape preserved)
- `src/module_system/` — rewrite with FileTable/FileId (~800 lines, 20+ string parsing ops)
- `src/error.rs` — rewrite as rich Diagnostic system (~700 lines, zero overlap with target)
- `src/main.rs` — decompose monolith into driver + CLI (~2K lines)
- `src/lsp/` — rewrite as thin consumer after compiler (~13K lines, 147 string parsing ops)
- LLVM system dependencies and `Cargo.toml` deps (huge build time win)

See **Appendix A** for the full gap analysis with evidence.

---

## Architecture

```
                          ZEN COMPILER PIPELINE

  source.zen ─→ [ Lexer ] ─→ [ Parser ] ─→ [ AST ]
                                              │
                                    [ Sema / Typechecker ]
                                      - type inference
                                      - generic monomorphization
                                      - comptime expansion (type_info → concrete code)
                                      - trait/behavior validation
                                              │
                                        [ Typed AST ]
                                              │
                                    [ C Codegen ]
                                      - structs → C structs
                                      - enums → tagged unions
                                      - methods → Type_method(self, args)
                                      - pattern match → switch/if
                                      - closures → function ptr + env struct
                                      - allocator vtable → C function pointers (already is)
                                              │
                                        [ .c / .h files ]
                                              │
                                    [ Build Driver ]
                                      - shells out to cc/gcc/clang
                                      - links system libs
                                              │
                                         [ binary ]
```

### Phase Breakdown

**Phase 1: Frontend (Lexer → Parser → AST)**
- Reuse/clean the existing lexer and parser
- Cleaner file layout (see below)
- All syntax decisions are settled — demo project is the spec

**Phase 2: Sema (Typechecker + Comptime)**
- Type inference and checking
- Generic monomorphization: `to_json<SensorReading>` → `to_json_SensorReading`
- Comptime expansion: `meta.type_info(T)` resolved at compile time, emits concrete branches
- Trait/behavior validation: verify `.implements` contracts
- Outputs a **Typed AST** where every node carries its resolved type (see below)
- Note: `build.zen` still needs comptime interpretation (evaluate at compile time to know what to build)

**Phase 3: Codegen (Typed AST → target)**
- `codegen/mod.rs` defines a `Backend` trait
- `codegen/c/` implements it — typed AST → `.c` / `.h` files
- If we ever want LLVM back, add `codegen/llvm/` implementing the same trait
- Direct translation, no IR passes
- See "C Codegen Mapping" section below

```rust
// codegen/mod.rs — backend interface
pub trait Backend {
    fn emit_program(&mut self, program: &TypedProgram, files: &FileTable) -> Result<(), Vec<Diagnostic>>;
    fn output_files(&self) -> Vec<OutputFile>;   // .c, .h files to write
}

// codegen/c/mod.rs
pub struct CBackend { ... }
impl Backend for CBackend { ... }

// future: codegen/llvm/mod.rs
// pub struct LlvmBackend { ... }
// impl Backend for LlvmBackend { ... }
```

**Phase 4: Build Driver**
- `zen build` reads `build.zen`, shells out to `cc`
- Links system libraries per platform
- Handles debug/release modes

---

## Target: The Demo Project

The rewrite's finish line is compiling `examples/demo_project/`.
Every phase is measured against: does `main.zen` + `build.zen` work?

---

## Typed AST

The Typed AST is sema's output and codegen's input — the most important interface
in the compiler. By the time codegen sees it, **every decision has been made**:
no generics, no unresolved types, no method lookups, no comptime.

### Principle: Codegen Is Mechanical

Codegen should never think. It walks the typed AST and emits C. If codegen needs
to make a decision (which function to call, what type something is), sema failed.

### Structure

```rust
// ─── Program ────────────────────────────────────────────────────────
struct TypedProgram {
    functions: Vec<TypedFunction>,    // all monomorphized
    types: Vec<TypedTypeDef>,         // all monomorphized structs/enums
    globals: Vec<TypedGlobal>,        // top-level constants, mutable globals
    entry_point: Option<String>,      // "main" if present
}

// ─── Functions ──────────────────────────────────────────────────────
struct TypedFunction {
    name: String,                     // mangled: "to_json_SensorReading"
    params: Vec<TypedParam>,
    return_type: Type,
    body: TypedBlock,
    defers: Vec<TypedExpr>,           // defer expressions in LIFO order
    span: Span,
}

struct TypedParam {
    name: String,
    ty: Type,
    span: Span,
}

// ─── Types ──────────────────────────────────────────────────────────
struct TypedTypeDef {
    name: String,                     // mangled: "Channel_SensorReading"
    kind: TypeDefKind,
    methods: Vec<TypedFunction>,      // all resolved methods
    span: Span,
}

enum TypeDefKind {
    Struct { fields: Vec<(String, Type)> },
    Enum { variants: Vec<TypedVariant> },
}

struct TypedVariant {
    name: String,
    tag: u32,                         // discriminant value
    payload: Option<Vec<(String, Type)>>,
}

// ─── Type Representation ────────────────────────────────────────────
// This is the resolved type — no generics, no inference variables.
enum Type {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Bool,
    Usize,
    Void,
    Str,                              // static string: { ptr, len }
    String,                           // heap string: { ptr, len, cap, alloc }
    Ptr(Box<Type>),                   // Ptr<T>
    MutPtr(Box<Type>),                // MutPtr<T>
    RawPtr(Box<Type>),                // RawPtr<u8>
    Array(Box<Type>, usize),          // [T; N]
    Slice(Box<Type>),                 // [T]
    Named(String),                    // struct/enum by mangled name
    FnPtr(Vec<Type>, Box<Type>),      // function pointer: (args) -> ret
    Error,                            // sentinel — sema couldn't resolve
}

// ─── Expressions ────────────────────────────────────────────────────
// Every expression carries its resolved type.
enum TypedExpr {
    IntLiteral(i64, Type, Span),
    FloatLiteral(f64, Type, Span),
    StrLiteral(String, Span),         // type is always Str
    BoolLiteral(bool, Span),
    Variable(String, Type, Span),

    FieldAccess {
        object: Box<TypedExpr>,
        field: String,
        ty: Type,
        span: Span,
    },

    // ALL calls resolved to concrete functions. No generics, no method lookup.
    // p.distance(other) already lowered to Point_distance(p, other)
    FunctionCall {
        function: String,             // mangled name
        args: Vec<TypedExpr>,
        return_type: Type,
        span: Span,
    },

    BinaryOp {
        left: Box<TypedExpr>,
        op: BinaryOp,
        right: Box<TypedExpr>,
        ty: Type,
        span: Span,
    },

    UnaryOp {
        op: UnaryOp,
        expr: Box<TypedExpr>,
        ty: Type,
        span: Span,
    },

    // The unified ? operator — disambiguated by sema into a specific kind
    Match {
        scrutinee: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
        ty: Type,                     // result type (Void if statement)
        kind: MatchKind,
        span: Span,
    },

    Cast {
        expr: Box<TypedExpr>,
        from_type: Type,
        to_type: Type,
        span: Span,
    },

    StructLiteral {
        type_name: String,            // mangled name
        fields: Vec<(String, TypedExpr)>,
        ty: Type,
        span: Span,
    },

    EnumVariant {
        type_name: String,
        variant: String,
        payload: Option<Vec<TypedExpr>>,
        ty: Type,
        span: Span,
    },

    ArrayLiteral {
        elements: Vec<TypedExpr>,
        element_type: Type,
        span: Span,
    },

    Index {
        array: Box<TypedExpr>,
        index: Box<TypedExpr>,
        element_type: Type,
        span: Span,
    },

    // Pointer operations — lowered from .ref(), .mut_ref(), .val
    Ref(Box<TypedExpr>, Type, Span),
    MutRef(Box<TypedExpr>, Type, Span),
    Deref(Box<TypedExpr>, Type, Span),

    // Closure — lowered to function + env struct (see Closures section)
    Closure {
        fn_name: String,              // generated: "__closure_0"
        env_type: String,             // generated: "__closure_env_0"
        captures: Vec<Capture>,
        ty: Type,                     // FnPtr type
        span: Span,
    },

    StringInterpolation {
        parts: Vec<StringPart>,
        allocator_expr: Box<TypedExpr>,
        span: Span,
    },

    // Compiler intrinsics — emitted directly as C builtins
    Intrinsic {
        name: String,                 // "raw_allocate", "syscall3", etc.
        args: Vec<TypedExpr>,
        return_type: Type,
        span: Span,
    },

    Assign {
        target: Box<TypedExpr>,
        value: Box<TypedExpr>,
        span: Span,
    },

    Block(TypedBlock),

    Error(Span),
}

// ─── Match (the ? operator) ─────────────────────────────────────────
// Sema resolves which kind of control flow ? represents.
enum MatchKind {
    Conditional,       // expr ? { body }           → if (expr) { body }
    ConditionalElse,   // expr ? | true {} | false {} → if/else
    WhileLoop,         // expr ? { body }            → while (expr) { body }
    EnumMatch,         // enum ? | Variant {} ...    → switch on tag
    ValueMatch,        // val ? | X {} | Y {}        → if/else chain
}

struct TypedMatchArm {
    pattern: TypedPattern,
    body: TypedBlock,
    span: Span,
}

enum TypedPattern {
    Bool(bool),
    EnumVariant { type_name: String, variant: String, bindings: Vec<(String, Type)> },
    Wildcard,
    Value(TypedExpr),
}

// ─── Closures ───────────────────────────────────────────────────────
struct Capture {
    name: String,
    ty: Type,
    by_ref: bool,      // true for MutPtr captures
}

enum StringPart {
    Literal(String),
    Expr(TypedExpr),
}
```

### What Sema Resolves (Codegen Never Decides)

| Before sema (untyped AST)           | After sema (typed AST)                          |
|--------------------------------------|-------------------------------------------------|
| `to_json(reading, alloc)`           | `FunctionCall { function: "to_json_SensorReading", ... }` |
| `classify(reading)`                  | `FunctionCall { function: "classify", return_type: Named("Alert"), ... }` |
| `p.distance(other)`                  | `FunctionCall { function: "Point_distance", args: [p, other], ... }` |
| `meta.type_info(T) ? \| Struct { }` | Only the Struct branch exists, with concrete field names |
| `alloc.allocate(1024)`              | `FunctionCall { function: "heap_sync_allocate", ... }` — wait, no. Vtable call stays as-is. |
| `x > 10 ? | true { body }`          | `Match { kind: Conditional, ... }`                                      |
| `i < 10 ? { body }`                 | `Match { kind: WhileLoop, ... }`                                        |
| `Channel<SensorReading>`            | `Named("Channel_SensorReading")` |
| `items.loop((item, i) { ... })`     | `FunctionCall` + `Closure { captures, fn_name, env_type }` |

**Correction on vtable calls**: `alloc.allocate(1024)` does NOT resolve to a concrete
function — it stays as a function pointer call through the vtable. Codegen emits
`self->allocate_fn(self->ctx, 1024)`. This is the whole point of the vtable pattern.

---

## Monomorphization

Generic functions and types are expanded into concrete versions during sema.
After monomorphization, there are no generics in the typed AST.

### When It Happens

Monomorphization runs during sema, interleaved with type checking:

```
1. Sema encounters call: to_json(reading, alloc)
2. Sema resolves reading: SensorReading
3. Sema looks up to_json<T>, substitutes T = SensorReading
4. Sema generates to_json_SensorReading (concrete function)
5. Sema type-checks the concrete function
6. Sema adds to_json_SensorReading to the typed AST
7. Sema replaces the call with to_json_SensorReading(reading, alloc)
```

### Name Mangling

All mangled names must be valid C identifiers.

```
Function mangling:
  to_json<SensorReading>            → to_json_SensorReading
  identity<i32>                     → identity_i32

Type mangling:
  Channel<SensorReading>            → Channel_SensorReading
  Result<i32, Error>                → Result_i32__Error
  Actor<CollectorMsg, Collector>    → Actor_CollectorMsg__Collector

Nested generics (flatten):
  Result<DynVec<String>, Error>     → Result_DynVec_String___Error

Rules:
  <        → _
  >        → (removed)
  ,<space> → __
  Ptr<T>   → Ptr_T
  MutPtr<T>→ MutPtr_T
```

### Method Mangling

Methods become free functions with the type name prefixed:

```
Point.distance        → Point_distance(Point* self, Point* other)
Collector.receive     → Collector_receive(Collector* self, ...)

With generics:
Channel<SensorReading>.send → Channel_SensorReading_send(...)
Actor<CollectorMsg, Collector>.ref → Actor_CollectorMsg__Collector_ref(...)
```

### Deduplication

```rust
// Sema maintains a map of already-monomorphized instantiations
monomorphized: HashMap<(String, Vec<Type>), String>

// Key: ("to_json", [Named("SensorReading")])
// Value: "to_json_SensorReading"

// If already exists, reuse. Don't generate twice.
```

### Recursion Guard

```rust
// Track the instantiation stack to detect infinite expansion
instantiation_stack: Vec<(String, Vec<Type>)>

// Before monomorphizing to_json<SensorReading>:
//   push ("to_json", [SensorReading]) onto stack
//   if already on stack → error: "infinite generic instantiation"
//   if stack depth > 64 → error: "generic instantiation depth limit"
//   monomorphize...
//   pop stack
```

### What Gets Monomorphized

| Generic construct              | Monomorphized output                       |
|-------------------------------|--------------------------------------------|
| `to_json<SensorReading>(...)` | Function `to_json_SensorReading`           |
| `Channel<SensorReading>.new()`| Struct `Channel_SensorReading` + all its methods |
| `Actor<CollectorMsg, Collector>.new()` | Struct + methods for this concrete pair |
| `Result<i32, Error>`          | Struct `Result_i32__Error` (tagged union)  |
| `meta.type_info(SensorReading)` | Comptime expanded: only the `Struct` branch with concrete field names/types |

### Comptime Expansion (Special Case)

`meta.type_info(T)` is resolved at compile time during monomorphization.
When `T = SensorReading`:

```
// Input (generic):
meta.type_info(T) ?
    | Struct { fields } { ... }
    | Enum { active_variant } { ... }
    | String { ... }
    | _ { ... }

// Output (after comptime expansion for T = SensorReading):
// Only the Struct branch remains, with concrete field info baked in:
{
    sb.append_char('{')
    // field 0: sensor_id: u32
    sb.append("\"sensor_id\":")
    sb.append(to_json_u32(value.sensor_id, alloc))
    sb.append_char(',')
    // field 1: temperature: f64
    sb.append("\"temperature\":")
    sb.append(to_json_f64(value.temperature, alloc))
    // ... etc
    sb.append_char('}')
}
```

Dead branches are eliminated. The runtime code has zero reflection overhead.

---

## Diagnostic System

One diagnostic type for the entire compiler. Every phase emits the same structure.
The LSP is just a consumer — if the diagnostics are good, the LSP is free.

### The Problem with the Current Compiler

Error reporting is ad-hoc. Each phase has its own error types, formatting, and reporting.
The LSP has to re-derive diagnostics by re-running analysis. Errors from different phases
don't compose. Adding a new warning means touching multiple files.

### Design: Single Diagnostic Type

```rust
// Every phase — lexer, parser, sema, codegen — emits Vec<Diagnostic>

struct Diagnostic {
    severity: Severity,       // Error, Warning, Hint, Info
    code: DiagnosticCode,     // E0001, W0042 — stable, searchable
    message: String,          // human-readable summary
    span: Span,               // primary location (file, line, col, length)
    labels: Vec<Label>,       // secondary locations with annotations
    context: Vec<ContextFrame>, // HOW we got here (call chain, instantiation, imports)
    notes: Vec<String>,       // additional help text ("did you mean...?")
    fix: Option<Fix>,         // suggested auto-fix (for LSP code actions)
}

struct Label {
    span: Span,
    message: String,          // annotation at this location
    style: LabelStyle,        // Primary, Secondary
}

// Context frames answer: "how did we get here?"
// They form a stack — innermost first, outermost last.
struct ContextFrame {
    span: Span,               // where this context is
    kind: ContextKind,        // what kind of context
    message: String,          // human-readable description
}

enum ContextKind {
    InFunction,               // "in function `process_readings`"
    InModule,                 // "in module `std.sync.channel`"
    InGenericInstantiation,   // "while instantiating `to_json<SensorReading>`"
    InTraitImpl,              // "while checking `Collector.implements(ActorBehavior)`"
    InImport,                 // "imported from `main.zen:3`"
    InMacroExpansion,         // "in expansion of `meta.type_info(T)`"
}

struct Fix {
    message: String,          // "Add missing type annotation"
    edits: Vec<TextEdit>,     // concrete text replacements
}

struct Span {
    file_id: FileId,          // index into file table
    start: u32,               // byte offset
    end: u32,                 // byte offset
}

enum Severity { Error, Warning, Hint, Info }
```

### Context Frames — Why Flat Spans Aren't Enough

Spans tell you WHERE. Context tells you HOW YOU GOT THERE.

**Without context** (flat span only):
```
error[E3001]: type mismatch: expected String, got i32
  --> stdlib/collections/string.zen:45:12
   |
45 |     sb.append(value.to_string())
   |               ^^^^^^^^^^^^^^^^^ expected String, got i32
```

You see the error but not WHY `value` is `i32`. Where was this function called?
Which generic instantiation produced this?

**With context frames**:
```
error[E3001]: type mismatch: expected String, got i32
  --> stdlib/collections/string.zen:45:12
   |
45 |     sb.append(value.to_string())
   |               ^^^^^^^^^^^^^^^^^ expected String, got i32
   |
   = while instantiating `to_json<i32>` (src/main.zen:66:20)
   = in function `to_json` called with T = i32
   = called from `main` (src/main.zen:232:44)
```

Now you know exactly how to fix it.

**More examples**:
```
error[E3012]: `Collector` doesn't implement required method `on_error`
  --> src/main.zen:171:1
   |
   = required by trait `ActorBehavior<CollectorMsg>` (stdlib/actor/actor.zen:91:1)
   = in `Collector.implements(ActorBehavior, { ... })` (src/main.zen:179:1)

error[E2005]: unexpected token `}`
  --> src/main.zen:204:1
   |
   = note: block opened at src/main.zen:179:89
   = in method `Collector.receive`
```

### How Each Phase Uses It

```
LEXER   → Diagnostic { code: E1001, "unterminated string literal",
                        span, context: [] }

PARSER  → Diagnostic { code: E2001, "expected '}' to close block",
                        labels: [{ span: open_brace, "block starts here" }],
                        context: [InFunction { "in function `main`" }] }

SEMA    → Diagnostic { code: E3001, "type mismatch: expected i32, got f64",
                        labels: [{ span: expr, "this is f64" },
                                 { span: param, "expected i32 here" }],
                        context: [InGenericInstantiation { "to_json<SensorReading>" },
                                  InFunction { "called from main" }],
                        fix: { "Add cast", [edit: "cast(expr, i32)"] } }

CODEGEN → Diagnostic { code: E4001, "cannot emit C for inline asm",
                        context: [InFunction { "in sensor_thread_fn" }] }
```

### Context Stack in the Compiler

Each phase maintains a context stack as it walks the AST:

```rust
struct DiagnosticEmitter {
    diagnostics: Vec<Diagnostic>,
    context_stack: Vec<ContextFrame>,   // pushed/popped as we enter/leave scopes
}

impl DiagnosticEmitter {
    fn push_context(&mut self, frame: ContextFrame) { self.context_stack.push(frame); }
    fn pop_context(&mut self) { self.context_stack.pop(); }

    fn emit(&mut self, severity: Severity, code: DiagnosticCode, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity, code, message: message.to_string(), span,
            labels: vec![],
            context: self.context_stack.clone(),  // snapshot current context
            notes: vec![],
            fix: None,
        });
    }
}

// Usage in sema:
emitter.push_context(ContextFrame {
    span: call_span,
    kind: ContextKind::InGenericInstantiation,
    message: format!("while instantiating `to_json<{concrete_type}>`"),
});
typecheck_function(monomorphized_fn, &mut emitter);
emitter.pop_context();
```

### Error Propagation Across Phases

All phases share one `Vec<Diagnostic>`. Each phase does its best with whatever
the previous phase produced — you get ALL the errors at once, not one at a time.

```
                   Vec<Diagnostic> (shared, accumulates across all phases)
                          │
  ┌─────────┐             │
  │  LEXER  │──errors────→├─ E1001: invalid escape sequence \q (main.zen:5)
  │         │──tokens────→│
  └─────────┘             │
       │ partial token    │
       │ stream           │
       ▼                  │
  ┌─────────┐             │
  │ PARSER  │──errors────→├─ E2005: expected '}' (main.zen:204)
  │         │──AST───────→│         context: [InFunction("Collector.receive")]
  └─────────┘             │
       │ partial AST      │
       │ (good decls +    │
       │  error nodes)    │
       ▼                  │
  ┌─────────┐             │
  │  SEMA   │──errors────→├─ E3001: type mismatch (stdlib/string.zen:45)
  │         │──typed AST─→│         context: [InGenericInstantiation("to_json<i32>"),
  └─────────┘             │                   InFunction("main")]
       │ partial typed    │
       │ AST              │
       ▼                  │
  ┌─────────┐             │
  │ CODEGEN │──errors────→├─ E4001: cannot emit C (main.zen:143)
  │         │──.c files──→│         context: [InFunction("sensor_thread_fn")]
  └─────────┘             │
                          ▼
                   CLI / LSP / CI renders ALL diagnostics
```

**Propagation strategy — each phase keeps going:**

| Phase   | On error...                                                     |
|---------|-----------------------------------------------------------------|
| Lexer   | Skip bad token, insert error token, keep lexing                 |
| Parser  | Skip to next declaration (sync on `}`/newline), keep parsing    |
| Sema    | Mark declaration as errored, skip it, keep checking the rest    |
| Codegen | **Stop** — can't produce a binary with holes                    |

**Key behaviors:**
- Parser errors in `Collector.receive` don't prevent sema from checking `main`
- Sema errors in `to_json<i32>` don't prevent codegen of `classify`
- A single `zen build` reports lexer + parser + sema errors together
- Only codegen is fatal — everything else is best-effort

**Error nodes in the AST:**

```rust
enum Expr {
    IntLiteral(i64, Span),
    BinaryOp { ... },
    // ...
    Error(Span),              // placeholder for expressions that failed to parse
}

enum Declaration {
    Function(Function),
    Struct(StructDef),
    // ...
    Error(Span),              // placeholder for declarations that failed to parse
}
```

Sema sees `Error` nodes and skips them (no cascading errors from bad parses).
Codegen sees `Error` nodes and refuses to emit (fatal).

### LSP Integration — Diagnostics ARE the LSP

The diagnostic system isn't a CLI thing that the LSP also uses.
It's the other way around: **every field in `Diagnostic` exists because the LSP needs it.**

```
  Diagnostic field        What it becomes in the editor
  ─────────────────────   ──────────────────────────────────────────────
  span                  → red/yellow squiggly underline on the exact range
  severity              → error (red) vs warning (yellow) vs hint (blue dots)
  message               → hover tooltip text
  code                  → clickable error code → links to docs page
  labels                → "related information" — secondary underlines in other
                           files/locations with their own annotations
  context               → breadcrumb trail shown in hover/tooltip:
                           "in to_json<SensorReading> → called from main"
  fix                   → lightbulb code action: "Add cast" → auto-applies edits
  fix.edits             → workspace edit — the LSP applies these text changes
  notes                 → shown below the error: "did you mean `f64`?"
```

**This means: if you design the diagnostic well, the LSP is zero extra work.**

```
                    ┌──────────────┐
  source.zen ──→    │  Compiler    │──→ Vec<Diagnostic>
                    │  Pipeline    │         │
                    └──────────────┘         │
                                             ├──→ CLI:  render with colors + source snippets
                                             ├──→ LSP:  1:1 map to LSP protocol
                                             └──→ CI:   JSON for tooling / GitHub annotations
```

The LSP server is thin — it doesn't re-analyze, it doesn't have its own error types.
It runs the compiler pipeline, gets `Vec<Diagnostic>`, maps each one:

```rust
fn to_lsp(d: &Diagnostic, files: &FileTable) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: span_to_lsp_range(d.span, files),
        severity: Some(match d.severity {
            Error   => DiagnosticSeverity::ERROR,
            Warning => DiagnosticSeverity::WARNING,
            Hint    => DiagnosticSeverity::HINT,
            Info    => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(d.code.to_string())),
        message: d.message.clone(),
        related_information: Some(
            d.labels.iter().chain(
                // context frames become related info too
                d.context.iter().map(|c| Label {
                    span: c.span,
                    message: c.message.clone(),
                    style: LabelStyle::Secondary,
                })
            ).map(|l| DiagnosticRelatedInformation {
                location: span_to_lsp_location(l.span, files),
                message: l.message.clone(),
            }).collect()
        ),
        ..Default::default()
    }
}

// Fix → CodeAction (the lightbulb)
fn to_code_action(d: &Diagnostic, files: &FileTable) -> Option<lsp_types::CodeAction> {
    d.fix.as_ref().map(|fix| lsp_types::CodeAction {
        title: fix.message.clone(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![to_lsp(d, files)]),
        edit: Some(fix_to_workspace_edit(&fix.edits, files)),
        ..Default::default()
    })
}
```

**What the user sees in their editor:**

```
  // In VS Code / Zed / Neovim with LSP:

  msg = "Hello, ${name}!"
        ~~~~~~~~~~~~~~~~~~~  ← red squiggly (span)

  [hover tooltip]
  error[E3042]: string interpolation requires an allocator in scope

  in function `greet` (src/main.zen:15)                    ← context frame

  did you mean to pass an `alloc: Allocator` parameter?    ← note

  💡 Quick fix: Add allocator parameter                    ← fix → code action
```

**Other LSP features that fall out of good diagnostics:**

| LSP feature          | Powered by                                           |
|----------------------|------------------------------------------------------|
| Error squiggles      | `span` + `severity`                                  |
| Hover error details  | `message` + `context` + `notes`                      |
| Problems panel       | Full `Vec<Diagnostic>` list                          |
| Quick fixes          | `fix` → `CodeAction`                                 |
| Go to related        | `labels` → secondary locations (click to jump)        |
| Error lens (inline)  | `message` displayed inline at `span.start` line      |
| Peek error           | `labels` + `context` → peek window with all locations |

**LSP features that need MORE than diagnostics (but share the same infra):**

| LSP feature          | Needs                                                |
|----------------------|------------------------------------------------------|
| Go to definition     | Sema's symbol table + spans                          |
| Hover type info      | Sema's type resolution + spans                       |
| Autocomplete         | Sema's scope/symbol table at cursor position         |
| Rename               | Sema's symbol table + all reference spans            |
| Find references      | Sema's symbol table + all reference spans            |

These all share `Span` and `FileTable`. The diagnostic system is the foundation —
build it right and everything else layers on top.

### File Table

All spans reference files by `FileId`, not by path string. One global table:

```rust
struct FileTable {
    files: Vec<SourceFile>,   // indexed by FileId
}

struct SourceFile {
    path: PathBuf,
    source: String,           // full source text (for error display)
    line_starts: Vec<u32>,    // byte offsets of each line start (for span → line/col)
}
```

Computing `line:col` from a byte offset is O(log n) binary search on `line_starts`.
This is computed once per file, cached, and shared across all phases.

### Error Codes

Namespaced by phase, stable across versions:

```
E1xxx — Lexer errors
E2xxx — Parser errors
E3xxx — Sema errors (type checking, trait resolution)
E4xxx — Codegen errors
W1xxx — Lexer warnings
W3xxx — Sema warnings (unused variables, etc.)
```

### What This Enables

- **LSP for free**: diagnostics are data, not side effects. LSP just maps them.
- **Code actions**: `Fix` field gives the LSP auto-fix suggestions directly.
- **Incremental**: re-run only the phase that changed, merge diagnostics.
- **CI/CD**: `zen check --format json` emits structured diagnostics for tooling.
- **Error index**: each code (E3001) can link to a documentation page explaining the error.
- **Multi-label errors**: "expected i32 here" + "but this expression is f64" in one diagnostic.

### File Layout

```
src/errors/
├── mod.rs           // Diagnostic, Severity, Label, Fix, Span, DiagnosticCode
├── codes.rs         // E1001..E4999 — all error/warning codes with descriptions
├── display.rs       // CLI formatting (colors, underlines, source snippets)
└── file_table.rs    // FileTable, SourceFile, FileId, span → line/col
```

This is built FIRST, before the lexer. Every other module depends on it.

---

## Code Organization Principle: Co-location

In the current compiler, type definitions are in one file, Display impls in another,
Debug in another, conversion methods somewhere else. This scatters related code across
the codebase and makes it hard to understand or modify a single concept.

**Rule: everything about a type lives with that type.**

```rust
// ast.rs — GOOD: definition + Display + Debug + helpers together

#[derive(Clone)]
pub enum Expr {
    IntLiteral(i64, Span),
    StringLiteral(String, Span),
    BinaryOp { left: Box<Expr>, op: BinaryOp, right: Box<Expr>, span: Span },
    // ...
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::IntLiteral(_, s) => s,
            Expr::StringLiteral(_, s) => s,
            Expr::BinaryOp { span, .. } => span,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::IntLiteral(v, _) => write!(f, "{v}"),
            Expr::StringLiteral(s, _) => write!(f, "\"{s}\""),
            Expr::BinaryOp { left, op, right, .. } => write!(f, "{left} {op} {right}"),
        }
    }
}

// NOT this:
//   ast/types.rs      — struct definitions
//   ast/display.rs    — Display impls (far from the structs)
//   ast/debug.rs      — Debug impls (even farther)
//   ast/helpers.rs    — span() methods (scattered)
```

**Apply this everywhere:**
- `token.rs` — Token enum + Display + is_keyword/is_operator helpers
- `ast.rs` — AST nodes + Display + span accessors + constructors
- `types.rs` — Type enum + Display + compatibility checks
- `diagnostic.rs` — Diagnostic struct + Display (CLI rendering)

If a file gets too large, split by **concept** (expressions vs statements vs declarations),
not by **trait** (Display vs Debug vs methods). Each concept file still has everything
about its types co-located.

---

## Language Spec

Derived from the demo project and stdlib. This is what the compiler must support.

### 1. Primitives

```
i8  i16  i32  i64
u8  u16  u32  u64
f32  f64
bool
usize
void
```

### 2. Strings (str vs String)

Two distinct string types. One needs an allocator, one doesn't.

```
// str — static, immutable, no allocator needed
// A pointer + length to read-only data. String literals produce this.
greeting = "hello"                   // type: str
multi = "line one\nline two"         // type: str (escape sequences)

// String — heap-allocated, growable, requires an allocator
name ::= String.new(alloc)          // type: String
name.append("world")
name.append_char('!')

// String interpolation produces String (needs alloc to concatenate)
msg = "Hello, ${name}!"             // type: String (alloc inferred from scope)

// Conversion
s: str = "static"
heap_s = String.from(s, alloc)      // str → String (copies into allocator)
view: str = heap_s.as_str()         // String → str (borrows, no copy)

// Function signatures make the cost clear
cheap = (label: str) void { ... }               // no allocation, just a view
expensive = (name: String, alloc: Allocator) void { ... }  // owns heap memory
```

**Why this matters:**
- `str` is zero-cost — passing string literals around never allocates
- `String` makes allocation explicit — you always see the allocator
- Pattern matching on enum variant names (`field.name`) returns `str`
- `to_json` returns `String` because it builds a new string on the heap
- C codegen: `str` → `struct { const char* ptr; size_t len; }`, `String` → growable buffer struct

**Safety model: trust the programmer (v1)**

`str` is an unowned view — like C's `const char*`. The compiler does NOT enforce
lifetime safety for `str`. This is an explicit design choice:

```
// SAFE: string literal → str has static lifetime (lives in .rodata)
greeting = "hello"                      // always safe

// SAFE: as long as the String is alive
name = String.from("world", alloc)
view: str = name.as_str()              // safe while `name` exists
io.println(view)

// UNSAFE: programmer's responsibility
get_name = (alloc: Allocator) str {
    s = String.from("hello", alloc)
    return s.as_str()                  // DANGER: s freed at scope exit
}
// Compiler does NOT catch this. It's a dangling pointer.
```

What we check:
- Can't modify through `str` (it's immutable)
- Can't cast `str` to `String` without an allocator

What we don't check:
- `str` outliving the `String` it borrows from (use-after-free)
- `str` pointing to freed memory

Zen is a systems language. For v1, we document the danger. Future: optional
lifetime annotations or a lint pass that warns on obvious patterns.

### 3. Variables & Mutability

```
x = 42                // immutable binding
y ::= 0               // mutable binding
y = y + 1             // reassignment (only for ::=)
z: i32 = 10           // explicit type annotation
```

### 4. Functions

```
// Named function
add = (a: i32, b: i32) i32 {
    return a + b
}

// Generic function — <T> binds to the name, not the assignment
identity<T> = (value: T) T {
    return value
}

// Generic with trait constraint
serialize<T: Serializable> = (value: T) String { ... }

// Call site mirrors definition: name<concrete>(args)
// identity<i32>(42)
// serialize<SensorReading>(reading)

// No return (void)
log = (msg: String) void {
    io.println(msg)
}
```

### 5. Structs

```
Point: {
    x: f64,
    y: f64,
}

// Construction
p = Point { x: 1.0, y: 2.0 }

// Access
p.x
```

### 6. Enums

```
// Simple (no payload)
Color:
    Red,
    Green,
    Blue

// With payloads
Result<T, E>:
    Ok: T,
    Err: E

// Mixed
Alert:
    Normal,
    Warning: { message: String },
    Critical: { code: i32, message: String }

// Construction
status = Alert.Warning { message: "hot" }
color = Color.Red
result = Result.Ok(42)
```

### 7. Pattern Matching (`?` / `|`)

The `?` operator is Zen's universal branch/match. No `if`, `match`, `switch` keywords.

**Disambiguation rules — the parser decides:**

| Syntax                          | Meaning        | C output                    |
|---------------------------------|----------------|-----------------------------|
| `expr ? { body }`              | **While loop** | `while (expr) { body; }`    |
| `expr ? \| arm \| arm`         | **Match/if**   | `if/switch`                 |
| `expr ? \| true { } \| false { }` | **If/else** | `if (expr) { } else { }`   |

The rule is simple: **`? {` (no `|`) is always a while loop. `? |` (with arms) is always a match.**

For a one-shot conditional (run once if true), use the match form:
```
// One-shot conditional — NOT a loop
index > 0 ? | true { sb.append_char(',') }

// While loop — repeats until condition is false
i < 10 ? {
    io.println("${i}")
    i = i + 1
}
```

This is a parser-level decision. No semantic analysis needed.

```
// Enum matching
alert ?
    | Normal              { io.println("ok") }
    | Warning { message } { io.println("warn: ${message}") }
    | Critical { code, message } {
        io.println("CRIT ${code}: ${message}")
    }

// Result matching
result ?
    | Ok(value) { use(value) }
    | Err(e)    { handle(e) }

// Option matching
opt ?
    | Some(v) { v }
    | None    { default }

// Wildcard
value ?
    | Specific { ... }
    | _        { fallback }

// Multi-arm (value matching)
os ?
    | Linux   { Link { libs: ["c", "m"] } }
    | Macos   { Link { frameworks: ["Foundation"] } }
    | Windows { Link { libs: ["kernel32"] } }
```

### 8. Loops

```
// While-style (condition ? { body } repeats while true)
i ::= 0
i < 10 ? {
    io.println("${i}")
    i = i + 1
}

// Iterator-style
items.loop((item, index) {
    io.println("${index}: ${item}")
})

// Range (aspirational)
range(0, 10).loop((i) { ... })
```

### 9. Methods

Methods are defined outside the struct using `Type.method` syntax.

```
Point.distance = (self: Ptr<Point>, other: Ptr<Point>) f64 {
    dx = self.val.x - other.val.x
    dy = self.val.y - other.val.y
    return sqrt(dx * dx + dy * dy)
}

// Static method (no self)
Point.origin = () Point {
    return Point { x: 0.0, y: 0.0 }
}

// Mutating method
Point.translate = (self: MutPtr<Point>, dx: f64, dy: f64) void {
    self.val.x = self.val.x + dx
    self.val.y = self.val.y + dy
}
```

### 10. Pointer Types

No `&` or `*` operators. Pointers are explicit types with methods.

```
Ptr<T>        // immutable pointer (like const T*)
MutPtr<T>     // mutable pointer (like T*)
RawPtr<u8>    // raw untyped pointer (like void*)

// Getting pointers from values
value.ref()       // → Ptr<T>
value.mut_ref()   // → MutPtr<T>

// Dereferencing
ptr.val           // → T (read through pointer)
ptr.val.field     // → access field through pointer

// Casting
cast(raw_ptr, Ptr<MyType>)
cast(ptr, MutPtr<MyType>)
```

### 11. Type Casting

```
cast(integer_value, f64)      // numeric conversion
cast(raw_ptr, Ptr<MyType>)    // pointer cast
cast(enum_val, i32)           // enum to int
```

### 12. Arrays

```
// Array literal
ids = [1, 2, 3, 4, 5]

// Typed array with size
handles ::= [MutPtr<ThreadHandle>; 3]

// Access
ids[0]

// Iteration
ids.loop((id, i) { ... })
```

### 13. Imports & Modules

```
// Destructuring import
{ io }                     = std
{ Heap, Arena, Allocator } = std.mem
{ Thread }                 = std.sync.thread
{ Channel }                = std.sync.channel

// Compiler builtins
{ meta }                   = @builtin

// Files are modules — no `mod`/`module`/`package` keywords
// Directory structure determines module path
```

### 14. Defer

```
alloc = Heap.sync()
@this.defer(alloc.deinit())    // runs at scope exit, LIFO order

ch ::= Channel.new(64, alloc)
@this.defer(ch.free())
```

### 15. Compile-Time Reflection

```
to_json<T> = (value: T, alloc: Allocator) String {
    meta.type_info(T) ?
        | Struct { fields } {
            fields.loop((field, index) {
                field.name          // field name as string
                field.get(value)    // get field value from instance
            })
        }
        | Enum { active_variant } {
            active_variant.name         // variant name
            active_variant.has_payload  // bool
            active_variant.payload      // payload value (if has_payload)
        }
        | String    { ... }
        | Integer   { ... }
        | Float     { ... }
        | Boolean   { ... }
        | _         { ... }
}
```

`meta.type_info(T)` is resolved at compile time. The compiler monomorphizes:
`to_json<SensorReading>` expands to code that only contains the `Struct` branch
with the concrete field names and types baked in.

### 16. Behaviors (the trait system)

There is **one** mechanism: `behavior`. No separate "trait" keyword.
`.implements`, `.extends`, `.requires` are operations on behaviors.

```
// Define a behavior — a contract of required methods
ActorBehavior<M>: behavior {
    receive: (self: MutPtr<Self>, ctx: MutPtr<ActorContext<M>>, msg: M) void
    on_start: (self: MutPtr<Self>, ctx: MutPtr<ActorContext<M>>) void
    on_stop: (self: MutPtr<Self>, ctx: MutPtr<ActorContext<M>>) void
}

Serializable: behavior {
    serialize: (self: Ptr<Self>, alloc: Allocator) String
    deserialize: (data: str, alloc: Allocator) Result<Self, Error>
}
```

**`.implements` — explicitly satisfy a behavior:**
```
SensorReading.implements(Serializable, {
    serialize = (self: Ptr<SensorReading>, alloc: Allocator) String {
        return to_json(self.val, alloc)
    }

    deserialize = (data: str, alloc: Allocator) Result<SensorReading, Error> {
        return from_json<SensorReading>(data, alloc)
    }
})

// Collector satisfies ActorBehavior by defining matching methods:
Collector.receive = (self: MutPtr<Collector>, ctx: MutPtr<ActorContext<CollectorMsg>>, msg: CollectorMsg) void {
    // ...
}
```

**`.requires` — compile-time assertion:**
```
SensorReading.requires(Serializable)
// Compile error if SensorReading doesn't implement Serializable
```

**`.extends` — behavior inheritance:**
```
PrettyPrint.extends(Serializable)
// PrettyPrint requires all of Serializable's methods + its own
```

**Generic constraints:**
```
process<T: Serializable> = (value: T, alloc: Allocator) void {
    json = value.serialize(alloc)
    io.println(json)
}
```

**C codegen**: behaviors are compile-time only. No vtable generated for behaviors.
Generic constraints are checked at monomorphization time — if `T = SensorReading`
and SensorReading doesn't implement Serializable, sema emits an error. At runtime,
the monomorphized code calls the concrete function directly.

### 17. Impl Blocks

Group methods on a type without a trait.

```
Point.impl = {
    new = (x: f64, y: f64) Point {
        return Point { x: x, y: y }
    }

    distance = (self: Ptr<Point>, other: Ptr<Point>) f64 {
        dx = self.val.x - other.val.x
        dy = self.val.y - other.val.y
        return sqrt(dx * dx + dy * dy)
    }
}
```

### 18. Allocators

Allocators are first-class values. The allocator determines execution mode.

```
// Sync — blocking I/O, OS threads
alloc = Heap.sync()

// Async — non-blocking I/O, task scheduler
alloc = Arena.async()

// Pass to any function — it doesn't care which
process_readings(channel, alloc)

// The SAME function:
//   with Heap.sync()   → recv() blocks the thread
//   with Arena.async() → recv() yields to the scheduler
```

The `Allocator` type is a struct with function pointers (manual vtable):

```
Allocator: {
    ctx: RawPtr<u8>,
    allocate_fn: (RawPtr<u8>, usize) RawPtr<u8>,
    deallocate_fn: (RawPtr<u8>, RawPtr<u8>, usize) void,
    mode_fn: (RawPtr<u8>) ExecutionMode,
    schedule_read_fn: ...,
    schedule_write_fn: ...,
    poll_fn: ...,
    wait_fn: ...,
}
```

This maps directly to C — it's already function pointers.

### 19. Concurrency Primitives

```
// Threads
thread = Thread.spawn(entry_fn, context, alloc)
thread.join()

// Channels (bounded MPSC/MPMC)
ch ::= Channel<T>.new(capacity, alloc)
ch.send(value)
ch.recv() ? | Some(v) { ... } | None { ... }
ch.close()

// Actors
actor ::= Actor<Msg, Behavior>.new(state, mailbox_size, alloc)
ref = actor.ref()
ref.send(message)
actor.run()    // blocks — run on dedicated thread
actor.stop()
actor.free()

// Supervision
supervisor ::= Supervisor.new(STRATEGY_ONE_FOR_ONE, max_restarts, window_sec, alloc)
supervisor.add_child(ChildSpec.permanent(id, start_fn, ctx))
supervisor.add_child(ChildSpec.transient(id, start_fn, ctx))
supervisor.start()
supervisor.stop()
```

### 20. Build System

`build.zen` is a Zen program that configures the build.

```
{Build, Builder, BuildConfig, BuildError, Package, Executable, Test, Link} = @builtin.build

build = (b:: Builder) Result<BuildConfig, BuildError> {

    packages = [
        Package { name: "std", path: "~/.zen/std" },
        Package { name: ".", path: "./src" },
        RemotePackage { name: "zen-js", url: "github.com/zenlang/zen-js", version: "0.1.0" },
    ]

    link = b.target.os ?
        | Linux   { Link { libs: ["c", "m"] } }
        | Macos   { Link { frameworks: ["Foundation"] } }
        | Windows { Link { libs: ["kernel32"] } }

    exe = Executable {
        name: "myapp",
        root_source_file: "src/main.zen",
        out_dir: "build/",
        packages: packages,
        link: link,
    }

    b.add(exe)

    b.is_release ?
        | true  { b.optimization(.O3); b.strip_symbols(true) }
        | false { b.optimization(.O0); b.debug_info(true) }

    return .Ok(b.config())
}
```

### 21. Operators

```
// Arithmetic
+  -  *  /

// Comparison
==  !=  <  >  <=  >=

// Logical
&&  ||  !

// Assignment
=     // immutable bind / reassign mutable
::=   // mutable bind

// NOTE: % (modulo) is being removed. Use a stdlib function instead.
```

### 22. Closures

Closures are anonymous functions passed to `.loop()`, `.map()`, etc.

```
// Syntax: (params) { body }
items.loop((item, i) {
    io.println("${i}: ${item}")
})

// Closures can capture variables from enclosing scope
alloc = Heap.sync()
items.loop((item, i) {
    json = to_json(item, alloc)     // captures `alloc`
    io.println(json)
})
```

**Capture rules:**
- Captures are **by value** (copy) by default
- `MutPtr` captures copy the pointer (the target is shared, not the pointer itself)
- Closures **cannot escape** their enclosing scope (stack closures only, v1)
- Cannot be stored in structs or returned from functions (v1)

**C lowering:**

Sema transforms each closure into a generated env struct + standalone function:

```
// Zen source:
alloc = Heap.sync()
items.loop((item, i) {
    json = to_json(item, alloc)
    io.println(json)
})

// C output:
typedef struct { Allocator alloc; } __closure_env_0;

void __closure_fn_0(__closure_env_0* __env, Item item, int64_t i) {
    zen_string json = to_json_Item(item, __env->alloc);
    io_println(json);
}

// at call site:
__closure_env_0 __env_0 = { .alloc = alloc };
Array_Item_loop(items, items_len, (void*)__closure_fn_0, &__env_0);
```

**`.loop()` signature (monomorphized):**
```
// The loop method takes a function pointer + opaque env pointer
Array_Item_loop = (arr: Ptr<Item>, len: usize, fn: (RawPtr<u8>, Item, i64) void, env: RawPtr<u8>) void
```

**Future (v2):** heap-allocated closures for callbacks stored in actors or returned from functions.

---

## Layering: Language vs Stdlib vs Compiler

Everything in Zen falls into one of three layers. Being explicit about this prevents
confusion about what users write, what's implementation detail, and what the Rust
compiler must provide.

```
┌─────────────────────────────────────────────────────────────────────┐
│  LAYER 1: Zen Language (user-facing)                                │
│  What users import and call. This is the public API.                │
│                                                                     │
│  Heap.sync()  Arena.async()  Allocator  Channel<T>  Thread  Actor  │
│  Supervisor   Task   Mmap   String   DynVec   HashMap   ...          │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 2: Zen Stdlib Internals (written in Zen, not user-facing)    │
│  Implementation details behind the Layer 1 API. Users don't touch.  │
│                                                                     │
│  HeapSync  ArenaAsync  heap_sync_allocate  arena_async_mode         │
│  AsyncPool  AsyncOp  Promise  TaskQueue  Scheduler  TaskContext     │
│  CompletionFn  context_switch  task_entry_trampoline  ...           │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 3: Compiler Intrinsics (implemented in Rust/C)               │
│  Primitives that can't be written in Zen. The compiler provides.    │
│                                                                     │
│  compiler.raw_allocate    compiler.syscall0..6    compiler.sizeof<T> │
│  compiler.atomic_load     compiler.memcpy         compiler.store<T>  │
│  compiler.int_to_ptr      compiler.ptr_to_int     compiler.trap      │
└─────────────────────────────────────────────────────────────────────┘
```

### Layer 1: Zen Language (User-Facing API)

This is what users import in their code. The public contract.

**Memory / Allocators**

| Symbol              | Import                          | What it is                                    |
|---------------------|---------------------------------|-----------------------------------------------|
| `Allocator`         | `std.mem` / `std.memory.allocator` | Struct with function pointers (manual vtable). The universal interface — every function that allocates takes this. |
| `ExecutionMode`     | `std.mem` / `std.memory.allocator` | Enum: `Sync`, `Async`. Queried via `alloc.mode()`. |
| `Heap`              | `std.mem` / `std.memory.heap`  | Namespace struct. Factory: `Heap.sync()` → `Allocator`. |
| `Arena`             | `std.memory.arena`              | Namespace struct. Factory: `Arena.async()` → `Allocator`. |
| `Mmap`              | `std.memory.mmap`               | Memory-mapped region. `Mmap.alloc(size)`, `Mmap.unmap()`. |

**What users write:**
```
alloc = Heap.sync()                  // get a blocking heap allocator
alloc = Arena.async()                // get an async arena allocator
ptr = alloc.allocate(1024)           // allocate through the vtable
alloc.deallocate(ptr, 1024)          // deallocate through the vtable
alloc.mode() ? | Sync { ... } | Async { ... }
```

Users never see `HeapSync`, never call `heap_sync_allocate`, never touch `compiler.raw_allocate`.
The `Allocator` struct is the boundary — everything behind it is implementation detail.

**Concurrency**

| Symbol              | Import                          | What it is                                    |
|---------------------|---------------------------------|-----------------------------------------------|
| `Thread`            | `std.sync.thread`               | `Thread.spawn(fn, ctx, alloc)` → OS thread    |
| `Channel<T>`        | `std.sync.channel`              | Bounded MPSC/MPMC. `send()`, `recv()`, `close()` |
| `Actor<M, B>`       | `std.actor`                     | Actor with mailbox. `Actor.new()`, `.ref()`, `.run()` |
| `ActorRef<M>`       | `std.actor`                     | Handle for sending messages: `ref.send(msg)` |
| `ActorBehavior<M>`  | `std.actor`                     | Behavior/trait: `receive`, `on_start`, `on_stop` |
| `ActorContext<M>`   | `std.actor`                     | Passed to actor methods: `ctx.stop()`, `ctx.self_ref()` |
| `Supervisor`        | `std.actor.supervisor`          | Manages actor lifecycle, restart strategies |
| `ChildSpec`         | `std.actor.supervisor`          | `.permanent()`, `.transient()`, `.temporary()` |

**What users write:**
```
ch ::= Channel<SensorReading>.new(64, alloc)
ch.send(reading)
ch.recv() ? | Some(v) { ... } | None { ... }

actor ::= Actor<Msg, MyBehavior>.new(state, 128, alloc)
ref = actor.ref()
ref.send(MyMsg.DoSomething)

supervisor ::= Supervisor.new(STRATEGY_ONE_FOR_ONE, 3, 60, alloc)
supervisor.add_child(ChildSpec.permanent(1, start_fn, ctx))
supervisor.start()
```

### Layer 2: Stdlib Internals (Written in Zen)

Implementation details that make Layer 1 work. Written in Zen, but not part of the
user-facing API. Users don't import these; they're internal to the stdlib modules.

**Allocator Backends**

| Symbol                    | File                      | What it does                                  |
|---------------------------|---------------------------|-----------------------------------------------|
| `HeapSync`                | `memory/heap.zen`         | Internal state struct (just a placeholder `i32`) |
| `heap_sync_allocate`      | `memory/heap.zen`         | Calls `compiler.raw_allocate(size)` — the vtable slot |
| `heap_sync_deallocate`    | `memory/heap.zen`         | Calls `compiler.raw_deallocate(ptr, size)` |
| `heap_sync_reallocate`    | `memory/heap.zen`         | Calls `compiler.raw_reallocate(ptr, old, new)` |
| `heap_sync_mode`          | `memory/heap.zen`         | Returns `ExecutionMode.Sync` |
| `heap_sync_schedule_read` | `memory/heap.zen`         | Blocking pread via `compiler.syscall3/4`, calls callback immediately |
| `heap_sync_schedule_write`| `memory/heap.zen`         | Blocking pwrite via `compiler.syscall3/4`, calls callback immediately |
| `heap_sync_poll/wait`     | `memory/heap.zen`         | No-op (sync operations complete immediately) |
| `ArenaAsync`              | `memory/arena.zen`        | Internal state: `base_ptr`, `arena_size`, `offset`, `_mode` |
| `arena_async_allocate`    | `memory/arena.zen`        | Bump allocation from arena memory (stub: uses raw_allocate) |
| `arena_async_deallocate`  | `memory/arena.zen`        | No-op — arenas free all at once |
| `arena_async_mode`        | `memory/arena.zen`        | Returns `ExecutionMode.Async` |
| `arena_async_schedule_*`  | `memory/arena.zen`        | Will use io_uring (stub: blocking fallback) |
| `arena_async_poll/wait`   | `memory/arena.zen`        | Will poll/wait on io_uring (stub: no-op) |
| `default_allocator()`     | `memory/heap.zen`         | Convenience: returns `Heap.sync()` |

**How `Heap.sync()` works internally:**
```
Heap.sync = () Allocator {
    ctx = compiler.raw_allocate(4)      // allocate HeapSync state
    compiler.store<i32>(ctx, 0)         // initialize placeholder

    return Allocator {
        ctx: ctx,
        allocate_fn: heap_sync_allocate,    // ← function pointer to stdlib fn
        deallocate_fn: heap_sync_deallocate,
        reallocate_fn: heap_sync_reallocate,
        mode_fn: heap_sync_mode,
        schedule_read_fn: heap_sync_schedule_read,
        schedule_write_fn: heap_sync_schedule_write,
        poll_fn: heap_sync_poll,
        wait_fn: heap_sync_wait,
    }
}
```

When user calls `alloc.allocate(1024)`, the call chain is:
```
alloc.allocate(1024)                // Layer 1: method on Allocator
  → self.allocate_fn(self.ctx, 1024)  // vtable dispatch
  → heap_sync_allocate(ctx, 1024)     // Layer 2: stdlib internal
    → compiler.raw_allocate(1024)     // Layer 3: compiler intrinsic
      → malloc(1024)                  // C runtime / OS
```

**Async Runtime**

| Symbol                  | File                        | What it does                                    |
|-------------------------|-----------------------------|-------------------------------------------------|
| `AsyncPool`             | `memory/async_pool.zen`     | I/O completion pool (stub: will be io_uring/epoll) |
| `AsyncOp`               | `memory/async_allocator.zen`| Tracks a pending async operation (id, callback, status) |
| `Promise`               | `memory/async_helpers.zen`  | Single async operation state (Pending/Completed/Failed) |
| `CompletionFn`          | `memory/allocator.zen`      | Function type: `(user_data: u64, result: i64) void` |
| `Task`                  | `concurrency/async/task.zen`| Stackful coroutine: own stack (mmap), context (registers), entry point |
| `TaskContext`           | `concurrency/async/task.zen`| Saved CPU state: rsp, rbp, rbx, r12-r15, rip |
| `TaskState`             | `concurrency/async/task.zen`| Enum: Created, Running, Suspended, Completed, Failed |
| `context_switch`        | `concurrency/async/task.zen`| Save/restore CPU registers (stub: needs asm) |
| `task_entry_trampoline` | `concurrency/async/task.zen`| First thing a new task executes — reads task ptr from stack |
| `Scheduler`             | `concurrency/async/scheduler.zen` | Manages task queues, context switching, I/O completion |
| `TaskQueue`             | `concurrency/async/scheduler.zen` | Ring buffer of task pointers |

**How async works (when fully implemented):**
```
1. Arena.async() returns Allocator with mode = Async
2. Channel.recv() checks alloc.mode():
   - Sync → futex_wait (block OS thread)
   - Async → task.suspend() (yield to scheduler)
3. Scheduler picks next runnable task, context_switch to it
4. When data arrives, scheduler resumes the suspended task
5. Channel.recv() returns the value — caller never knew it yielded
```

This is why `process_readings` works with both `Heap.sync()` and `Arena.async()` —
the allocator's mode drives the blocking behavior, not the function's code.

### Layer 3: Compiler Intrinsics (Implemented in Rust/C)

These are primitives that **cannot** be written in Zen. The compiler emits them
directly as LLVM IR (current) or C code (rewrite target).

**Memory**

| Intrinsic                 | Current impl (Rust/LLVM)         | C backend target                |
|---------------------------|----------------------------------|---------------------------------|
| `compiler.raw_allocate(size)` | `call @malloc`                | `malloc(size)`                  |
| `compiler.raw_deallocate(ptr, size)` | `call @free`           | `free(ptr)`                     |
| `compiler.raw_reallocate(ptr, old, new)` | `call @realloc`    | `realloc(ptr, new)`             |
| `compiler.memcpy(dest, src, n)` | `@llvm.memcpy`              | `memcpy(dest, src, n)`          |
| `compiler.memset(dest, val, n)` | `@llvm.memset`              | `memset(dest, val, n)`          |
| `compiler.sizeof<T>()`   | Resolved at compile time          | `sizeof(T_c_type)`             |

**Pointer / Cast**

| Intrinsic                 | Current impl                     | C backend target                |
|---------------------------|----------------------------------|---------------------------------|
| `compiler.int_to_ptr(addr)` | `inttoptr i64 %addr to i8*`   | `(void*)(intptr_t)addr`         |
| `compiler.ptr_to_int(ptr)`  | `ptrtoint i8* %ptr to i64`    | `(int64_t)(intptr_t)ptr`        |
| `compiler.store<T>(ptr, val)` | `store T %val, T* %ptr`      | `*(T_c*)ptr = val`              |
| `compiler.load<T>(ptr)`    | `load T, T* %ptr`              | `*(T_c*)ptr`                    |
| `compiler.raw_ptr_cast(ptr)` | `bitcast`                     | `(target_type*)ptr`             |

**Atomics**

| Intrinsic                 | Current impl                     | C backend target                |
|---------------------------|----------------------------------|---------------------------------|
| `compiler.atomic_load(ptr)`  | `load atomic ... seq_cst`     | `atomic_load(ptr)`              |
| `compiler.atomic_store(ptr, val)` | `store atomic ... seq_cst` | `atomic_store(ptr, val)`       |
| `compiler.atomic_add(ptr, val)` | `atomicrmw add`              | `atomic_fetch_add(ptr, val)`    |
| `compiler.atomic_sub(ptr, val)` | `atomicrmw sub`              | `atomic_fetch_sub(ptr, val)`    |
| `compiler.atomic_cas(ptr, exp, new)` | `cmpxchg`               | `atomic_compare_exchange_strong(ptr, &exp, new)` |
| `compiler.atomic_xchg(ptr, val)` | `atomicrmw xchg`           | `atomic_exchange(ptr, val)`     |

**Syscalls**

| Intrinsic                 | Current impl                     | C backend target                |
|---------------------------|----------------------------------|---------------------------------|
| `compiler.syscall0..6(nr, ...)` | Inline asm `syscall`        | `syscall(nr, ...)` or inline asm |

**Other**

| Intrinsic                 | Current impl                     | C backend target                |
|---------------------------|----------------------------------|---------------------------------|
| `compiler.trap()`        | `@llvm.trap`                      | `__builtin_trap()`              |
| `compiler.breakpoint()`  | `@llvm.debugtrap`                 | `__builtin_debugtrap()`         |

### Why This Layering Matters

**For the C backend rewrite:**
- Layer 1 and 2 are pure Zen — the C backend just compiles them like any other code
- Layer 3 is the only thing the C backend must special-case (emit C builtins/intrinsics)
- The intrinsic list above is **exhaustive** — that's all the compiler needs to provide

**For users:**
- They only see Layer 1. `Heap.sync()` returns an `Allocator`, done.
- They never think about `HeapSync` or `heap_sync_allocate` or `compiler.raw_allocate`
- The vtable pattern means new allocators (Pool, Slab, etc.) are just new Zen code in stdlib

**For stdlib developers:**
- Layer 2 is where new allocator strategies go — write a new `pool.zen`, wire up function pointers
- The only hard constraint: you can only call Layer 3 intrinsics for things Zen can't express
- Everything else is regular Zen code that the compiler handles normally

### Current Allocator Ecosystem

```
Layer 1 (user-facing)         Layer 2 (stdlib internal)           Layer 3 (compiler)
─────────────────────         ──────────────────────────          ──────────────────
Heap.sync() ─────────────→ HeapSync + heap_sync_* fns ──────→ compiler.raw_allocate
                              (blocking I/O via syscalls)          compiler.syscall3

Arena.async() ───────────→ ArenaAsync + arena_async_* fns ──→ compiler.raw_allocate
                              (bump alloc, async I/O stubs)       compiler.syscall3
                              AsyncPool (io_uring stub)
                              Task (stackful coroutines)
                              Scheduler (task queue + switch)

Allocator.allocate() ────→ vtable dispatch to whichever
Allocator.mode()            backend was constructed
Allocator.schedule_read()

Mmap.alloc() ────────────→ Mmap struct + methods ───────────→ compiler.syscall6 (SYS_MMAP)

(future)
Pool.fixed(size, n) ────→ PoolFixed + pool_fixed_* fns ───→ compiler.raw_allocate
Slab.new(sizes) ─────────→ SlabAlloc + slab_* fns ─────────→ Mmap.alloc (Layer 2)
Heap.async() ────────────→ HeapAsync + heap_async_* fns ──→ compiler.raw_allocate
                              (io_uring integration)              compiler.syscall6 (io_uring_enter)
```

---

## C Codegen Mapping

How each Zen construct translates to C:

| Zen                                | C output                                         |
|------------------------------------|--------------------------------------------------|
| `Point: { x: f64, y: f64 }`       | `typedef struct { double x; double y; } Point;`  |
| `Alert: Normal, Warning: { msg }`  | Tagged union: `struct { int tag; union { ... } }` |
| `Point.distance = (self, other)`   | `double Point_distance(Point* self, Point* other)` |
| `to_json<SensorReading>(...)`      | `char* to_json_SensorReading(SensorReading v, Allocator a)` |
| `meta.type_info(T) ? \| Struct`    | Sema expands → only the matching branch emitted  |
| `x > 10 ? \| true { a } \| false { b }` | `if (x > 10) { a; } else { b; }`           |
| `result ? \| Ok(v) { } \| Err(e)`  | `if (result.tag == OK) { v = result.ok; } else ...` |
| `Allocator` vtable                 | Already function pointers — emit as-is           |
| `ch.send(msg)`                     | `Channel_SensorReading_send(&ch, msg)`            |
| `@this.defer(x.free())`           | Emit `x_free()` calls before every return path   |
| `value.ref()`                      | `&value` (in C)                                  |
| `value.mut_ref()`                  | `&value` (in C, pointer to non-const)            |
| `ptr.val`                          | `*ptr` or `ptr->field`                           |
| `cast(x, f64)`                     | `(double)x`                                      |
| `str` (static string)              | `zen_str { const char* ptr; size_t len; }`       |
| `String` (heap string)             | `zen_string { char* ptr; size_t len, cap; Allocator alloc; }` |
| `String` interpolation `"${x}"`   | String builder calls (needs allocator)            |
| `"literal"` (str)                  | `(zen_str){ .ptr = "literal", .len = 7 }`        |
| `items.loop((item, i) { ... })`    | `for (int i = 0; i < len; i++) { ... }`          |
| `i < 10 ? { body }`               | `while (i < 10) { body; }`                       |
| closure `(x) { x + captured }`    | env struct + standalone function + call-site init |
| `behavior` constraint              | Compile-time only — no runtime representation    |

### Forward Declarations

C requires types/functions to be declared before use. The C backend emits output in this order:

```c
// 1. Forward declarations for ALL structs/enums (handles mutual recursion)
typedef struct Point Point;
typedef struct Alert Alert;
typedef struct Channel_SensorReading Channel_SensorReading;

// 2. Full struct/enum definitions (now all type names are known)
struct Point { double x; double y; };
struct Alert { int tag; union { ... } data; };

// 3. Function forward declarations (handles mutual recursion between functions)
double Point_distance(const Point* self, const Point* other);
Alert classify(SensorReading reading);
zen_string to_json_SensorReading(SensorReading value, Allocator alloc);

// 4. Function bodies
double Point_distance(const Point* self, const Point* other) { ... }
// ...
```

This is a simple two-pass emit: collect all type/function names first, then emit bodies.
No topological sort needed — forward declarations handle all ordering.

### Debug Info: `#line` Directives

Without `#line` directives, `gdb` steps through generated C code — useless for users.
With them, `gdb` steps through the original `.zen` source:

```c
#line 40 "src/main.zen"
Alert classify(SensorReading reading) {
#line 41 "src/main.zen"
    if (reading.temperature > 35.0) {
#line 42 "src/main.zen"
        return (Alert){ .tag = ALERT_CRITICAL, .data.critical = { .code = 1, ... }};
    }
}
```

**Rules:**
- Emit `#line` before every statement that maps to a Zen source line
- Use the span's file path and line number from the FileTable
- `cc -g` then generates DWARF info pointing to `.zen` files, not `.c` files
- Compile with `-g` in debug mode, omit `#line` in release mode (cleaner C output)

---

## Complete Feature Checklist

### Must Have (required by demo project)

- [ ] Diagnostic system — Span, FileTable, Diagnostic, error codes, display
- [ ] Lexer — tokens, spans, string interpolation
- [ ] Parser — all syntax below
- [ ] Structs with fields
- [ ] Enums with mixed payloads
- [ ] Functions (named, generic)
- [ ] Pattern matching (`?` / `|`)
- [ ] Methods (`Type.method = ...`)
- [ ] Impl blocks (`Type.impl = { ... }`)
- [ ] Variables (`=` immutable, `::=` mutable)
- [ ] Type inference
- [ ] Explicit type annotations
- [ ] Pointer types (`Ptr<T>`, `MutPtr<T>`, `RawPtr<u8>`)
- [ ] `.ref()`, `.mut_ref()`, `.val`
- [ ] `cast(expr, Type)`
- [ ] Arrays (literal, typed+sized, indexing, `.loop`)
- [ ] `str` (static string) vs `String` (heap string)
- [ ] String interpolation
- [ ] Defer (`@this.defer(...)`)
- [ ] Imports (`{ X, Y } = module`)
- [ ] Module system (files are modules)
- [ ] Generics + monomorphization
- [ ] Compile-time type reflection (`meta.type_info`)
- [ ] Behaviors (`Name: behavior { ... }`)
- [ ] Traits (`.implements`, `.requires`, `.extends`)
- [ ] Closures / lambdas (for `.loop` callbacks)
- [ ] Build system (`build.zen` evaluation)
- [ ] C codegen
- [ ] Build driver (shell out to cc)

### Should Have (stdlib needs)

- [ ] Raw syscall wrappers (`compiler.syscall*`)
- [ ] Atomic operations (`compiler.atomic_*`)
- [ ] Memory intrinsics (`compiler.memcpy`, `compiler.sizeof<T>`)
- [ ] Raw pointer arithmetic (`compiler.int_to_ptr`, `compiler.ptr_to_int`)
- [ ] Function pointers as values
- [ ] Global mutable state (`COUNTER :: i64 = 0`)
- [ ] Behavior default implementations

### Nice to Have (future)

- [ ] Async/await integration with allocator mode
- [ ] io_uring backend for Arena.async
- [ ] Package manager (RemotePackage resolution)
- [ ] LSP server
- [ ] Incremental compilation
- [ ] REPL

---

## Proposed File Layout (New)

```
zen/
├── Cargo.toml              # minimal deps — no LLVM
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # library root
│   │
│   ├── frontend/
│   │   ├── mod.rs
│   │   ├── lexer.rs        # tokenizer
│   │   ├── token.rs        # token types + Display
│   │   ├── parser.rs       # recursive descent parser
│   │   ├── ast.rs          # AST node types + Display + Debug (co-located)
│   │   └── span.rs         # source locations
│   │
│   ├── sema/
│   │   ├── mod.rs
│   │   ├── types.rs        # type representation
│   │   ├── checker.rs      # type checking
│   │   ├── inference.rs    # type inference
│   │   ├── monomorph.rs    # generic monomorphization
│   │   ├── comptime.rs     # compile-time expansion (type_info etc.)
│   │   ├── traits.rs       # trait/behavior resolution
│   │   └── typed_ast.rs    # AST annotated with types
│   │
│   ├── codegen/
│   │   ├── mod.rs           # shared codegen trait / interface
│   │   └── c/
│   │       ├── mod.rs
│   │       ├── emitter.rs   # typed AST → C source
│   │       ├── types.rs     # Zen types → C type strings
│   │       ├── patterns.rs  # pattern match → if/switch
│   │       ├── builtins.rs  # compiler.* intrinsics → C equivalents
│   │       └── runtime.h    # minimal runtime (tagged unions, defer macro)
│   │   # future: llvm/ can slot in here with the same interface
│   │
│   ├── build/
│   │   ├── mod.rs
│   │   ├── driver.rs       # orchestrate: parse → sema → codegen → cc
│   │   ├── build_zen.rs    # evaluate build.zen
│   │   └── cc.rs           # shell out to C compiler
│   │
│   ├── modules/
│   │   ├── mod.rs
│   │   ├── resolver.rs     # import resolution
│   │   └── package.rs      # package map, stdlib location
│   │
│   ├── errors/
│   │   ├── mod.rs
│   │   └── diagnostic.rs   # error formatting, spans
│   │
│   └── intrinsics.rs       # builtin functions registry
│
├── stdlib/                  # Zen standard library (unchanged)
│   ├── std.zen
│   ├── memory/
│   ├── collections/
│   ├── concurrency/
│   ├── io/
│   └── ...
│
├── examples/
│   └── demo_project/       # THE target — must compile
│       ├── build.zen
│       └── main.zen
│
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    ├── sema_tests.rs
    ├── codegen_c_tests.rs   # C backend specific
    └── integration/         # end-to-end: .zen → binary → run → check output
```

---

## Implementation Order

### Sprint 0: Foundations

Goal: error infrastructure that every phase builds on

- Diagnostic type (Span, Label, Fix, Severity, DiagnosticCode)
- FileTable (FileId, SourceFile, line_starts, span → line/col)
- CLI display (colors, underlines, source snippets, multi-label)
- JSON output mode for CI
- AST Display/Debug traits — co-located with AST node definitions
- Test: can create, format, and render diagnostics

### Sprint 1: Parse the demo

Goal: `zen parse examples/demo_project/main.zen` → AST dump

- Lexer (tokens, string interpolation, spans) — emits `Vec<Diagnostic>` on errors
- Parser (structs, enums, functions, generics, `?`/`|`, methods, imports)
- AST pretty printer (Display impls alongside node definitions)
- Test: parse main.zen and build.zen without errors

### Sprint 2: Type check the demo

Goal: `zen check examples/demo_project/main.zen` → no type errors

This is the hardest sprint. Break it into sub-milestones:

**2a. Type representation + inference (foundations)**
- `Type` enum (primitives, Named, Ptr, MutPtr, RawPtr, FnPtr, Array, Str, String)
- Typed AST node types (TypedExpr, TypedFunction, TypedProgram)
- Type inference engine: literals, variables, field access, binary ops
- Scope/symbol table: track variable types, function signatures
- Test: typecheck simple functions with primitives and structs

**2b. Method resolution + Typed AST emission**
- Method lookup: `Type.method` → `Type_method` free function
- Self parameter resolution (Ptr<T> vs MutPtr<T>)
- Static methods (no self parameter)
- Typed AST construction: every expression annotated with resolved type
- Test: typecheck methods on Point, Channel, etc.

**2c. Monomorphization**
- Monomorphization engine: substitute type params, generate concrete functions
- Name mangling (see Monomorphization section)
- Deduplication map: don't generate same instantiation twice
- Recursion guard: detect infinite generic expansion
- Monomorphize generic types too (Channel<T> → Channel_SensorReading struct + methods)
- Test: to_json<SensorReading> produces concrete function

**2d. Comptime expansion**
- `meta.type_info(T)` expansion: inspect concrete type, emit only matching branch
- Field iteration: generate concrete code for each struct field
- Dead branch elimination: remove Enum/String/Integer branches when T is Struct
- Test: to_json<SensorReading> produces code with concrete field names

**2e. Behavior validation + closures**
- `.implements` checking: verify all required methods exist with correct signatures
- `.requires` assertion: compile error if behavior not satisfied
- Generic constraints: `<T: Serializable>` checked at monomorphization site
- Closure lowering: generate env struct + standalone function for each closure
- Capture analysis: determine which variables are captured, by value or ref
- Test: Collector satisfies ActorBehavior, closures in .loop() work

### Sprint 3: Emit C

Goal: `zen build examples/demo_project/` → compiles C → produces binary

- C emitter for each AST node type
- Runtime header (tagged unions, defer, string builder)
- Pattern match compilation
- Monomorphized function emission
- Build driver (evaluate build.zen, invoke cc)
- Test: demo binary runs, output matches expected

### Sprint 4: Stdlib on C

Goal: stdlib compiles through the C backend

- Syscall wrappers → inline asm or libc calls
- Atomic operations → C11 atomics
- Channel, Thread, Actor all work
- Test: full demo with threads + actors runs correctly

---

## Appendix A: Gap Analysis Audit Results (2026-02-11)

A team of 4 agents audited the entire codebase (~54K lines, 182 files). This appendix
documents what was found and informs the rewrite plan.

### Executive Summary

| Module | Lines | String Parsing Leaks | Critical Bugs | Salvage Rating |
|--------|-------|---------------------|---------------|----------------|
| Lexer | ~1.2K | 0 (but has \x01/\x02 interpolation hack) | 0 | 3/5 |
| Parser | ~7K | 4 (generic→string, method→string, module→string) | 1 (`str` type rejected) | 2/5 |
| Typechecker | ~7K | 16 (13 critical + 3 moderate) | 7 (discriminant bug, unsound defaults) | 2/5 |
| Codegen (LLVM) | ~12K | 15+ | ~2000 lines doing sema's job | 2/5 |
| Comptime | ~1.9K | 1 | 0 | **4/5** |
| Module System | ~811 | 20+ | 1 (silent failures) | 2/5 |
| Build System | ~518 | 3 | 0 | **4/5** |
| Intrinsics | ~514 | 0 | 0 | **5/5** |
| Error System | ~700 | 1 (detailed_message string matching) | architectural (flat enum) | 1/5 |
| AST | ~2.8K | 0 | architectural (no Typed AST, no Error nodes) | 2/5 |
| main.rs | ~2K | 3 | architectural (monolith, reimplements lexer) | 1/5 |
| LSP | ~13.2K | **147** | architectural (duplicates compiler) | 2/5 |
| Tests | ~4.2K | 0 | coverage gaps | 3/5 |
| **Total** | **~54K** | **~210** | | |

### The Root Cause: Generics Encoded as Strings

The single biggest problem across the entire codebase originates from one function:
`parse_generic_type_args_to_string()` in `parser/expressions/literals.rs:183`.

This function bakes generic type arguments INTO identifier name strings (e.g., the parser
produces `"HashMap<i32, String>"` instead of `name="HashMap", type_args=[I32, String]`).

Every downstream consumer must then re-parse these strings:

```
Parser: "HashMap<i32, String>" stored in name field
    ↓
Typechecker: name.contains('<') → parse_type_from_string()     [13 instances]
Codegen:     func.find('<') → manual slice parsing              [15 instances]
Module System: module_path.split('.') → reconstruct from string [20 instances]
LSP: 147 instances of string parsing for completions/hover/etc
```

The `FunctionCall` AST node already HAS a `type_args: Vec<AstType>` field, but the parser
sometimes fills `name` with generics baked in AND leaves `type_args` empty. Fixing this at
the parser level eliminates ~50 downstream parsing leaks.

### Critical Bugs Found

**1. types_compatible() discriminant comparison (typechecker/validation.rs:301)**

```rust
if std::mem::discriminant(expected) == std::mem::discriminant(actual) {
    return true;  // BUG: Struct{name:"A"} == Struct{name:"B"}
}
```

Compares enum discriminants, not values. Any two structs are "compatible". Any two enums
are "compatible". This means the typechecker silently accepts assignments between
completely different struct types.

**2. Generic defaults to I32 (typechecker/scope.rs:101)**

Unresolved type parameter `T` silently defaults to `I32`, `E` defaults to `String`.
This hides real type errors in any generic code.

**3. Pattern binding types wrong (typechecker/pattern_binding.rs:120)**

In `Point { x, y }` destructuring, both `x` and `y` get type `Point` instead of their
actual field types. Struct field types are never looked up.

**4. Codegen maintains shadow type system (~2000 lines)**

Three parallel type tracking mechanisms in codegen:
- `inference.rs` (1116 lines) — full type inference, defaults unknowns to I32
- `GenericTypeTracker` — scoped generic type mapping stack
- `generic_type_context: HashMap<String, AstType>` — yet another parallel tracker

These exist because codegen receives an untyped AST and must re-derive types.

**5. Method resolution is a 10-strategy waterfall (typechecker/inference/calls.rs)**

Method calls try 10 strategies in order. Strategy 6 (hardcoded stdlib types) intercepts
before Strategy 8 (trait methods), meaning stdlib types can never use proper trait
resolution. The order is load-bearing and fragile.

**6. Closure params default to I32 (typechecker/inference/closures.rs:22)**

All untyped closure parameters get type I32. Only works for `.loop()` callbacks by
coincidence. Any other closure use (callbacks, map, filter) gets wrong types.

**7. String interpolation uses \x01/\x02 markers (lexer.rs:627)**

Instead of proper InterpolationStart/InterpolationEnd tokens (which exist but are dead
code!), the lexer embeds raw expression text in `\x01`..`\x02` markers. The parser then
re-creates a new Lexer+Parser for each interpolated expression with no span context.

### Module-by-Module Salvage Assessment

**KEEP AS-IS (rating 4-5)**

- **intrinsics.rs** (5/5) — Cleanest module. Macro-based registration, constants for
  magic strings, WellKnownTypes registry. No changes needed.
- **comptime/** (4/5) — Clean architecture, minimal string parsing, well-separated
  concerns. Gaps (float/I64 arithmetic, for-in) are additive. Meta-programming framework
  is solid. Enhance incrementally.
- **build_system/** (4/5) — PackageMap design is sound. Comptime execution path works.
  Only 3 string parsing instances.

**KEEP CONCEPTS, REWRITE CODE (rating 2-3)**

- **Lexer** (3/5) — Core tokenization is solid. Needs: proper interpolation tokens,
  accept `str` as type, clean up dead code (InterpolationStart/End, next_token).
- **Parser** (2/5) — Coverage of the spec is nearly complete (20/22 sections). But
  generic-args-as-strings is baked into the architecture. Closure parsing duplicated 4x.
  Module paths must become Vec<String>. The parser's structure is salvageable but its
  output format (the untyped AST) needs fundamental changes.
- **AST** (2/5) — Expression/Statement/Declaration enum shapes are reasonable. But needs:
  Typed AST layer, Error nodes for recovery, clean up messy AstType variants
  (StaticLiteral, StdModule), explicit generic type args everywhere.
- **Tests** (3/5) — Behavioral test SCENARIOS are reusable (they describe what the
  language should do). But the Rust test code depends on current APIs. Keep the test
  intent, rewrite the harness.

**FULL REWRITE (rating 1-2)**

- **Typechecker** (2/5) — The 4-pass pipeline structure is sound (collect → resolve →
  infer → check). But types_compatible() is broken, generics default to I32, no
  monomorphization, no Typed AST output, method resolution is a 10-strategy waterfall.
  The pipeline shape survives; everything else is rewritten.
- **Codegen** (2/5) — ~2000 of 12000 lines are type inference that belongs in sema.
  Switching to C backend means this is deleted entirely. stdlib_codegen/compiler.rs
  patterns (syscall wrappers, memory ops) inform the C backend design.
- **Module System** (2/5) — 20+ string parsing ops, load-parse-cache duplicated 4x, no
  FileTable/FileId, ModuleResolver is dead code. The caching concept (FNV-1a hashing,
  LRU eviction) is good but the implementation is string-soup.
- **Error System** (1/5) — Zero overlap with REWRITE.md Diagnostic spec. Flat enum,
  no codes, no severity, no labels, no context frames, no Fix. `detailed_message()`
  does runtime string matching on error messages. Complete rewrite.
- **main.rs** (1/5) — 2K-line monolith. Reimplements lexer via character scanning,
  parser via regex, scope analysis via string matching. load-parse-typecheck duplicated
  5x. Decompose into driver + CLI modules.
- **LSP** (2/5) — 13K lines, 147 string parsing ops. Duplicates compiler internals
  because the compiler doesn't expose structured data. Once Typed AST + Diagnostics +
  symbol table exist, the LSP becomes a thin adapter layer. Rewrite after compiler.

### Final Verdict: Clean Rewrite

The evidence overwhelmingly supports a clean rewrite. The problems are not individual
bugs that can be fixed incrementally — they are architectural:

1. **No Typed AST** — Adding one means rewriting typechecker output AND codegen input.
   That's 19K lines of fundamental restructuring, not patching.

2. **String parsing is systemic** — 210+ leaks all originating from the parser encoding
   generics into strings. Fixing the parser output format cascades to every consumer.

3. **Codegen doing sema** — 2000 lines of type inference in codegen can't be moved to
   sema until Typed AST exists. Circular dependency with #1.

4. **Error system is orthogonal** — Zero overlap between current flat enum and target
   rich Diagnostics. Every module that produces errors must change.

Incremental cleanup would touch every module anyway. A clean rewrite starts from the
correct architecture (Typed AST, structured generics, rich Diagnostics) and never
accumulates the string-parsing debt.

### What the Rewrite Preserves

```
KEEP INTACT:
  stdlib/                     — All .zen source (this IS the spec)
  examples/                   — Demo project + tutorials
  docs/REWRITE.md             — This document
  src/intrinsics.rs           — Clean, well-structured, no changes needed

KEEP AS FOUNDATION (enhance, don't rewrite):
  src/comptime/               — Clean architecture, add float/I64/for-in
  src/build_system/mod.rs     — PackageMap concept

KEEP INTENT (rewrite implementation):
  tests/behavioral_tests.rs   — Test scenarios describe expected behavior
  src/ast/                    — Enum shapes inform Typed AST design
  Lexer tokenization          — Token types are correct, fix interpolation

DELETE:
  src/codegen/llvm/           — Replaced by C codegen
  src/typechecker/            — Rewrite to produce Typed AST
  src/module_system/          — Rewrite with FileTable/FileId
  src/error.rs                — Rewrite as Diagnostic system
  src/main.rs                 — Decompose into driver + CLI
  src/lsp/                    — Rewrite as thin consumer after compiler
  src/formatting.rs           — 1 string parsing leak, trivial to redo
  src/name_utils.rs           — Exists only to work around string-encoded types
```
