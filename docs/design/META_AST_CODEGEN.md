# Meta-Driven AST Code Generation: A Self-Hosting Architecture for Zen

## Executive Summary

This document proposes a fundamental architectural pattern for the Zen compiler: **exposing the AST as Zen-visible types through the `meta` module**, enabling code generators to be written **as Zen programs** rather than Rust modules. This approach transforms backend development from low-level compiler engineering into high-level metaprogramming, makes the compiler incrementally self-hosting, and opens codegen to user extension.

**The Core Pattern:**
```zen
{ meta } = @std

// AST walking via meta.type_info() + pattern matching
walk = (node: ASTNode, emitter: Emitter) void {
    meta.type_info(node).kind ?
        | Function(f) {
            emitter.emit_fn_start(f.name, f.args)
            f.body.loop((stmt) { walk(stmt, emitter) })
            emitter.emit_fn_end()
        }
        | BinaryOp(op) {
            walk(op.left, emitter)
            emitter.emit(op.operator)
            walk(op.right, emitter)
        }
        | ...
}

// C backend - just Zen code implementing Emitter
CEmitter: { buffer:: StringBuilder, indent:: u32 }
CEmitter.implements(Emitter, { /* emit methods */ })
```

This eliminates the need for separate Rust modules for C, JS, Python, WASM backends. Instead, each backend is a Zen program that walks the AST and emits target code.

---

## 1. The Core Insight

### The Fundamental Realization

**AST nodes are already typed data structures.** Looking at `/home/ubuntu/zenlang/src/ast/expressions.rs`:

```rust
pub enum Expression {
    Integer32(i32),
    BinaryOp { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
    FunctionCall { name: String, type_args: Vec<AstType>, args: Vec<Expression> },
    QuestionMatch { scrutinee: Box<Expression>, arms: Vec<MatchArm> },
    // ... 40+ variants
}
```

This is **already a sum type (enum)**. If we expose this through `meta.type_info()`, Zen code can:

1. **Inspect the AST structure** - `meta.type_info(expr).kind` returns the variant
2. **Pattern match on node types** - Use `?` operator to dispatch on Expression variants
3. **Access fields** - `meta.fields(expr)` gives access to `left`, `right`, `op`, etc.
4. **Walk the tree recursively** - Since fields contain child AST nodes, recursion is natural

### Why This Works

The pieces already exist in Zen:

- **`meta.type_info(T) -> TypeInfo`** (LANGUAGE_SPEC.zen lines 629-636): Returns type structure
- **TypeKind enum** (lines 638-645): Already has `Struct`, `Enum`, `Function` variants
- **Pattern matching with `?`**: Zen's native dispatch mechanism
- **Behaviors as interfaces**: The visitor pattern maps cleanly to Zen behaviors
- **Strong typing**: Type mismatches are caught at compile time

The compiler already has:

- Complete AST definitions in `/home/ubuntu/zenlang/src/ast/`
- A comptime interpreter in `/home/ubuntu/zenlang/src/comptime/mod.rs`
- The `ComptimeValue` enum that can hold values at compile time

**What's missing**: A `ComptimeValue::ASTNode` variant that holds AST references, allowing AST nodes to be first-class comptime values.

### The Self-Hosting Implication

Once AST nodes are Zen-visible:

- **Code generators become user code** - No compiler hacking required
- **Optimization passes are Zen functions** - Walk AST, return transformed AST
- **The parser could be Zen** - Return AST nodes as Zen values
- **Bootstrap path is clear**: Zen → C codegen (written in Zen) → gcc → binary

This is how Zig, Rust, and other self-hosting languages evolve. The difference: **Zen does it through metaprogramming, not special compiler phases.**

---

## 2. What the Meta Module Must Expose

### 2.1 The Core AST Types

Based on `/home/ubuntu/zenlang/src/ast/`, these Rust types must become Zen-visible:

#### Expression Enum (40+ variants)

```zen
// Exposed as Zen enum through meta.type_info()
Expression:
    Integer8: i8,
    Integer16: i16,
    Integer32: i32,
    Integer64: i64,
    Unsigned8: u8,
    Unsigned16: u16,
    Unsigned32: u32,
    Unsigned64: u64,
    Float32: f32,
    Float64: f64,
    Boolean: bool,
    String: String,
    Identifier: String,
    Unit: void,

    BinaryOp: {
        left: Ptr<Expression>,
        op: BinaryOperator,
        right: Ptr<Expression>,
    },

    FunctionCall: {
        name: String,
        type_args: []AstType,
        args: []Expression,
    },

    QuestionMatch: {
        scrutinee: Ptr<Expression>,
        arms: []MatchArm,
    },

    StructLiteral: {
        name: String,
        fields: [](String, Expression),
    },

    ArrayLiteral: []Expression,

    ArrayIndex: {
        array: Ptr<Expression>,
        index: Ptr<Expression>,
    },

    EnumVariant: {
        enum_name: String,
        variant: String,
        payload: Option<Ptr<Expression>>,
    },

    MemberAccess: {
        object: Ptr<Expression>,
        member: String,
    },

    MethodCall: {
        object: Ptr<Expression>,
        method: String,
        type_args: []AstType,
        args: []Expression,
    },

    Block: []Statement,

    Loop: { body: Ptr<Expression> },

    CollectionLoop: {
        collection: Ptr<Expression>,
        param: (String, Option<AstType>),
        index_param: Option<(String, Option<AstType>)>,
        body: Ptr<Expression>,
    },

    Closure: {
        params: [](String, Option<AstType>),
        return_type: Option<AstType>,
        body: Ptr<Expression>,
    },

    Return: Ptr<Expression>,
    Break: { label: Option<String>, value: Option<Ptr<Expression>> },
    Continue: { label: Option<String> },

    Range: {
        start: Ptr<Expression>,
        end: Ptr<Expression>,
        inclusive: bool,
    },

    // ... (all 40+ variants)
```

#### Statement Enum (13 variants)

```zen
Statement:
    Expression: {
        expr: Expression,
        span: Option<Span>,
    },

    Return: {
        expr: Expression,
        span: Option<Span>,
    },

    VariableDeclaration: {
        name: String,
        type_: Option<AstType>,
        initializer: Option<Expression>,
        is_mutable: bool,
        declaration_type: VariableDeclarationType,
        span: Option<Span>,
    },

    VariableAssignment: {
        name: String,
        value: Expression,
        span: Option<Span>,
    },

    PointerAssignment: {
        pointer: Expression,
        value: Expression,
        span: Option<Span>,
    },

    Loop: {
        kind: LoopKind,
        label: Option<String>,
        body: []Statement,
        span: Option<Span>,
    },

    Break: {
        label: Option<String>,
        span: Option<Span>,
    },

    Continue: {
        label: Option<String>,
        span: Option<Span>,
    },

    ComptimeBlock: {
        statements: []Statement,
        span: Option<Span>,
    },

    Defer: {
        statement: Ptr<Statement>,
        span: Option<Span>,
    },

    DestructuringImport: {
        names: []String,
        source: Expression,
        span: Option<Span>,
    },

    Block: {
        statements: []Statement,
        span: Option<Span>,
    }
```

#### Declaration Enum (12 variants)

```zen
Declaration:
    Function: {
        name: String,
        type_params: []TypeParameter,
        args: [](String, AstType),
        return_type: AstType,
        body: []Statement,
        is_varargs: bool,
        is_public: bool,
    },

    ExternalFunction: {
        name: String,
        args: []AstType,
        return_type: AstType,
        is_varargs: bool,
    },

    Struct: {
        name: String,
        type_params: []TypeParameter,
        fields: []StructField,
        methods: []Function,
        span: Option<Span>,
    },

    Enum: {
        name: String,
        type_params: []TypeParameter,
        variants: []EnumVariant,
        methods: []Function,
        required_traits: []String,
        span: Option<Span>,
    },

    Behavior: {
        name: String,
        type_params: []TypeParameter,
        methods: []BehaviorMethod,
    },

    Trait: {
        name: String,
        type_params: []TypeParameter,
        methods: []TraitMethod,
        span: Option<Span>,
    },

    TraitImplementation: {
        type_name: String,
        trait_name: String,
        type_params: []TypeParameter,
        methods: []Function,
    },

    ImplBlock: {
        type_name: String,
        type_params: []TypeParameter,
        methods: []Function,
    },

    ComptimeBlock: []Statement,

    Constant: {
        name: String,
        value: Expression,
        type_: Option<AstType>,
        span: Option<Span>,
    },

    ModuleImport: {
        alias: String,
        module_path: String,
        span: Option<Span>,
    },

    TypeAlias: {
        name: String,
        type_params: []TypeParameter,
        target_type: AstType,
        span: Option<Span>,
    }
```

#### AstType Enum (24 variants)

```zen
AstType:
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    Usize,
    F32, F64,
    Bool,
    StaticLiteral,  // Internal string literals
    StaticString,   // User-facing StringLiteral
    Void,

    Slice: Ptr<AstType>,

    FixedArray: {
        element_type: Ptr<AstType>,
        size: usize,
    },

    Function: {
        args: []AstType,
        return_type: Ptr<AstType>,
    },

    FunctionPointer: {
        param_types: []AstType,
        return_type: Ptr<AstType>,
    },

    Struct: {
        name: String,
        fields: [](String, AstType),
    },

    Enum: {
        name: String,
        variants: []EnumVariant,
    },

    Ref: Ptr<AstType>,

    Range: {
        start_type: Ptr<AstType>,
        end_type: Ptr<AstType>,
        inclusive: bool,
    },

    Generic: {
        name: String,
        type_args: []AstType,
    },

    EnumType: { name: String },
    StdModule
```

#### Pattern Enum (10 variants)

```zen
Pattern:
    Literal: Expression,
    Identifier: String,

    Struct: {
        name: String,
        fields: [](String, Pattern),
    },

    EnumVariant: {
        enum_name: String,
        variant: String,
        payload: Option<Ptr<Pattern>>,
    },

    EnumLiteral: {
        variant: String,
        payload: Option<Ptr<Pattern>>,
    },

    Wildcard,
    Or: []Pattern,
    Tuple: []Pattern,

    Range: {
        start: Ptr<Expression>,
        end: Ptr<Expression>,
        inclusive: bool,
    },

    Binding: {
        name: String,
        pattern: Ptr<Pattern>,
    },

    Type: {
        type_name: String,
        binding: Option<String>,
    },

    Guard: {
        pattern: Ptr<Pattern>,
        condition: Ptr<Expression>,
    }
```

#### Supporting Structs

```zen
// From declarations.rs
StructField: {
    name: String,
    type_: AstType,
    is_mutable: bool,
    default_value: Option<Expression>,
}

Function: {
    name: String,
    type_params: []TypeParameter,
    args: [](String, AstType),
    return_type: AstType,
    body: []Statement,
    is_varargs: bool,
    is_public: bool,
}

TypeParameter: {
    name: String,
    constraints: []TraitConstraint,
}

TraitConstraint: {
    trait_name: String,
}

BehaviorMethod: {
    name: String,
    params: []Parameter,
    return_type: AstType,
}

Parameter: {
    name: String,
    type_: AstType,
    is_mutable: bool,
}

// From expressions.rs
MatchArm: {
    pattern: Pattern,
    guard: Option<Expression>,
    body: Expression,
}

BinaryOperator:
    Add, Subtract, Multiply, Divide, Modulo,
    Equals, NotEquals,
    LessThan, GreaterThan, LessThanEquals, GreaterThanEquals,
    And, Or,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    ShiftLeft, ShiftRight

// From statements.rs
VariableDeclarationType:
    InferredImmutable,  // =
    InferredMutable,    // ::=
    ExplicitImmutable,  // : T
    ExplicitMutable     // :: T

LoopKind:
    Infinite,
    Condition: Expression

// From types.rs
EnumVariant: {
    name: String,
    payload: Option<AstType>,
}
```

### 2.2 The Program Root

```zen
Program: {
    declarations: []Declaration,
    module_path: String,
}
```

This is the top-level entry point that codegen receives.

### 2.3 The ASTNode Union Type

To enable generic AST traversal, we need a union type:

```zen
ASTNode:
    Expr: Expression,
    Stmt: Statement,
    Decl: Declaration,
    Type: AstType,
    Pattern: Pattern
```

This allows writing generic walkers:

```zen
walk = (node: ASTNode, emitter: Emitter) void {
    node ?
        | Expr(e) { walk_expression(e, emitter) }
        | Stmt(s) { walk_statement(s, emitter) }
        | Decl(d) { walk_declaration(d, emitter) }
        | Type(t) { walk_type(t, emitter) }
        | Pattern(p) { walk_pattern(p, emitter) }
}
```

### 2.4 Meta Module API Extensions

Beyond what's in LANGUAGE_SPEC.zen (lines 616-794), we need:

```zen
// Get TypeInfo for AST nodes (enabling introspection)
meta.type_info(node: ASTNode) -> TypeInfo

// Access fields of an AST node
meta.fields(node: ASTNode) -> []FieldInfo

// Access enum variant data
meta.variant_data(node: ASTNode) -> Option<VariantData>

// Check if an AST node is a specific variant
meta.is_variant(node: ASTNode, variant_name: String) -> bool

// Get the variant name of an enum value
meta.variant_name(node: ASTNode) -> String

// Traverse a collection of AST nodes
meta.children(node: ASTNode) -> []ASTNode
```

These are **not new language features** - they're applications of existing `meta.type_info()` to AST types.

---

## 3. The Visitor/Walker Pattern

### 3.1 The Basic Walker

The walker is a recursive function that dispatches on AST node types:

```zen
{ meta } = @std

walk_expression = (expr: Expression, emitter: Emitter) void {
    expr ?
        | Integer32(val) {
            emitter.emit_int(val)
        }
        | String(val) {
            emitter.emit_string(val)
        }
        | Identifier(name) {
            emitter.emit_identifier(name)
        }
        | BinaryOp(op) {
            walk_expression(op.left.val, emitter)
            emitter.emit_operator(op.op)
            walk_expression(op.right.val, emitter)
        }
        | FunctionCall(call) {
            emitter.emit_call_start(call.name)
            call.args.loop((arg, i) {
                i > 0 ? { emitter.emit_separator() }
                walk_expression(arg, emitter)
            })
            emitter.emit_call_end()
        }
        | QuestionMatch(match_) {
            emitter.emit_match_start()
            walk_expression(match_.scrutinee.val, emitter)
            match_.arms.loop((arm) {
                emitter.emit_arm_start()
                walk_pattern(arm.pattern, emitter)
                arm.guard ? | Some(guard) {
                    emitter.emit_guard_start()
                    walk_expression(guard, emitter)
                    emitter.emit_guard_end()
                }
                walk_expression(arm.body, emitter)
                emitter.emit_arm_end()
            })
            emitter.emit_match_end()
        }
        | Block(stmts) {
            emitter.emit_block_start()
            stmts.loop((stmt) { walk_statement(stmt, emitter) })
            emitter.emit_block_end()
        }
        | Loop(loop_) {
            emitter.emit_loop_start()
            walk_expression(loop_.body.val, emitter)
            emitter.emit_loop_end()
        }
        | CollectionLoop(loop_) {
            emitter.emit_collection_loop_start(loop_.collection, loop_.param)
            walk_expression(loop_.body.val, emitter)
            emitter.emit_collection_loop_end()
        }
        | MethodCall(call) {
            walk_expression(call.object.val, emitter)
            emitter.emit_method_call(call.method, call.args)
        }
        | Closure(closure) {
            emitter.emit_closure_start(closure.params, closure.return_type)
            walk_expression(closure.body.val, emitter)
            emitter.emit_closure_end()
        }
        | Return(expr) {
            emitter.emit_return_start()
            walk_expression(expr.val, emitter)
            emitter.emit_return_end()
        }
        | Break(brk) {
            emitter.emit_break(brk.label, brk.value)
        }
        | Continue(cont) {
            emitter.emit_continue(cont.label)
        }
        | _ {
            // Unhandled expression type - error or warning
            @std.io.eprintln("Unhandled expression variant")
        }
}

walk_statement = (stmt: Statement, emitter: Emitter) void {
    stmt ?
        | Expression(expr_stmt) {
            walk_expression(expr_stmt.expr, emitter)
            emitter.emit_statement_end()
        }
        | Return(ret) {
            emitter.emit_return_start()
            walk_expression(ret.expr, emitter)
            emitter.emit_return_end()
        }
        | VariableDeclaration(decl) {
            emitter.emit_var_decl(decl.name, decl.type_, decl.is_mutable)
            decl.initializer ? | Some(init) {
                emitter.emit_assignment()
                walk_expression(init, emitter)
            }
            emitter.emit_statement_end()
        }
        | Loop(loop_) {
            loop_.kind ?
                | Infinite {
                    emitter.emit_infinite_loop_start(loop_.label)
                }
                | Condition(cond) {
                    emitter.emit_while_loop_start(loop_.label)
                    walk_expression(cond, emitter)
                }
            emitter.emit_loop_body_start()
            loop_.body.loop((stmt) { walk_statement(stmt, emitter) })
            emitter.emit_loop_body_end()
        }
        | _ { /* Handle other statement types */ }
}

walk_declaration = (decl: Declaration, emitter: Emitter) void {
    decl ?
        | Function(func) {
            emitter.emit_function_start(func.name, func.args, func.return_type)
            func.body.loop((stmt) { walk_statement(stmt, emitter) })
            emitter.emit_function_end()
        }
        | Struct(struct_) {
            emitter.emit_struct_start(struct_.name, struct_.type_params)
            struct_.fields.loop((field) {
                emitter.emit_field(field.name, field.type_, field.is_mutable)
            })
            emitter.emit_struct_end()
        }
        | Enum(enum_) {
            emitter.emit_enum_start(enum_.name, enum_.type_params)
            enum_.variants.loop((variant) {
                emitter.emit_variant(variant.name, variant.payload)
            })
            emitter.emit_enum_end()
        }
        | _ { /* Handle other declaration types */ }
}
```

### 3.2 The Emitter Behavior (Interface)

The emitter is a **behavior** - Zen's equivalent of traits/interfaces:

```zen
Emitter: {
    // Expressions
    emit_int: (self, val: i32) void,
    emit_string: (self, val: String) void,
    emit_identifier: (self, name: String) void,
    emit_operator: (self, op: BinaryOperator) void,

    emit_call_start: (self, name: String) void,
    emit_call_end: (self) void,
    emit_separator: (self) void,

    emit_match_start: (self) void,
    emit_match_end: (self) void,
    emit_arm_start: (self) void,
    emit_arm_end: (self) void,
    emit_guard_start: (self) void,
    emit_guard_end: (self) void,

    emit_block_start: (self) void,
    emit_block_end: (self) void,

    emit_loop_start: (self) void,
    emit_loop_end: (self) void,

    emit_collection_loop_start: (self, collection: Expression, param: (String, Option<AstType>)) void,
    emit_collection_loop_end: (self) void,

    emit_method_call: (self, method: String, args: []Expression) void,

    emit_closure_start: (self, params: [](String, Option<AstType>), return_type: Option<AstType>) void,
    emit_closure_end: (self) void,

    emit_return_start: (self) void,
    emit_return_end: (self) void,

    emit_break: (self, label: Option<String>, value: Option<Ptr<Expression>>) void,
    emit_continue: (self, label: Option<String>) void,

    // Statements
    emit_statement_end: (self) void,
    emit_var_decl: (self, name: String, type_: Option<AstType>, is_mutable: bool) void,
    emit_assignment: (self) void,

    emit_infinite_loop_start: (self, label: Option<String>) void,
    emit_while_loop_start: (self, label: Option<String>) void,
    emit_loop_body_start: (self) void,
    emit_loop_body_end: (self) void,

    // Declarations
    emit_function_start: (self, name: String, args: [](String, AstType), return_type: AstType) void,
    emit_function_end: (self) void,

    emit_struct_start: (self, name: String, type_params: []TypeParameter) void,
    emit_struct_end: (self) void,
    emit_field: (self, name: String, type_: AstType, is_mutable: bool) void,

    emit_enum_start: (self, name: String, type_params: []TypeParameter) void,
    emit_enum_end: (self) void,
    emit_variant: (self, name: String, payload: Option<AstType>) void,

    // Output
    output: (self) String,
}
```

### 3.3 State Machine Management

Emitters carry mutable state through the walk:

```zen
EmitterState: {
    // Buffer for accumulating output
    buffer:: StringBuilder,

    // Indentation tracking
    indent_level:: u32,
    indent_string:: String,

    // Context flags
    in_function:: bool,
    in_loop:: bool,
    in_match_arm:: bool,

    // Scope tracking
    scope_depth:: u32,
    current_scope_vars:: Vec<String>,

    // Target-specific state
    forward_decls:: Vec<String>,  // C needs forward declarations
    import_statements:: Vec<String>,  // JS/Python imports
    used_stdlib_functions:: HashSet<String>,  // Track stdlib dependencies

    // Type context
    current_function_return_type:: Option<AstType>,
    generic_substitutions:: HashMap<String, AstType>,
}

// Helper methods for state management
emit_indent = (state: EmitterState) void {
    i = 0
    loop i < state.indent_level {
        state.buffer.append(state.indent_string)
        i = i + 1
    }
}

push_scope = (state: EmitterState) void {
    state.scope_depth = state.scope_depth + 1
}

pop_scope = (state: EmitterState) void {
    state.scope_depth = state.scope_depth - 1
}
```

### 3.4 Composable Walkers

Since walkers are just functions, they can be chained:

```zen
// Optimization pass: constant folding
optimize = (program: Program) Program {
    optimizer = ConstantFoldingOptimizer.new()
    optimized_decls = program.declarations.map((decl) {
        optimize_declaration(decl, optimizer)
    })
    return Program { declarations: optimized_decls, module_path: program.module_path }
}

// Linting pass: check for unused variables
lint = (program: Program) []LintWarning {
    linter = UnusedVariableLinter.new()
    walk_program(program, linter)
    return linter.warnings()
}

// Codegen pass: generate C code
to_c = (program: Program) String {
    emitter = CEmitter.new()
    walk_program(program, emitter)
    return emitter.output()
}

// Compose: optimize → lint → codegen
compile = (source: String) Result<String, []Error> {
    program = parse(source).raise()
    optimized = optimize(program)
    warnings = lint(optimized)
    warnings.loop((w) { @std.io.println(w.message) })
    c_code = to_c(optimized)
    return .Ok(c_code)
}
```

---

## 4. Type Mapping Tables

### 4.1 Primitives

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `i8` | `int8_t` | `number` | `int` | Signed 8-bit |
| `i16` | `int16_t` | `number` | `int` | Signed 16-bit |
| `i32` | `int32_t` | `number` | `int` | Signed 32-bit (JS loses precision > 2^53) |
| `i64` | `int64_t` | `BigInt` | `int` | Signed 64-bit (JS needs BigInt) |
| `u8` | `uint8_t` | `number` | `int` | Unsigned 8-bit |
| `u16` | `uint16_t` | `number` | `int` | Unsigned 16-bit |
| `u32` | `uint32_t` | `number` | `int` | Unsigned 32-bit |
| `u64` | `uint64_t` | `BigInt` | `int` | Unsigned 64-bit |
| `usize` | `size_t` | `number/BigInt` | `int` | Platform-dependent |
| `f32` | `float` | `number` | `float` | 32-bit float |
| `f64` | `double` | `number` | `float` | 64-bit float |
| `bool` | `_Bool` or `bool` (C99) | `boolean` | `bool` | Boolean |
| `void` | `void` | `undefined` | `None` | Unit type |

### 4.2 Strings

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `StringLiteral` | `const char*` | `string` | `str` | Compile-time string (immutable, no allocator) |
| `String` (stdlib) | `zen_string_t*` or struct | `string` | `str` | Heap-allocated string (has allocator field) |

**C struct for String:**
```c
typedef struct {
    uint8_t* data;
    uint64_t len;
    uint64_t capacity;
    zen_allocator_t* allocator;
} zen_string_t;
```

### 4.3 Collections

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `[T]` | `zen_slice_t<T>` | `T[]` (view) | `list[T]` (view) | Slice (ptr + len, no ownership) |
| `[T; N]` | `T[N]` | `T[]` | `tuple[T, ...]` | Fixed-size array (stack) |
| `Vec<T, N>` | `zen_vec_t<T, N>` | `T[]` | `list[T]` | Fixed-capacity vector |
| `DynVec<T>` | `zen_dynvec_t<T>` | `T[]` | `list[T]` | Dynamic vector (resizable) |
| `HashMap<K, V>` | `zen_hashmap_t<K, V>` | `Map<K, V>` | `dict[K, V]` | Hash map |
| `HashSet<T>` | `zen_hashset_t<T>` | `Set<T>` | `set[T]` | Hash set |

**C struct for DynVec:**
```c
typedef struct {
    void* data;
    uint64_t len;
    uint64_t capacity;
    zen_allocator_t* allocator;
} zen_dynvec_t;
```

### 4.4 Pointers

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `Ptr<T>` | `const T*` | N/A (reference) | N/A (reference) | Immutable pointer |
| `MutPtr<T>` | `T*` | N/A (reference) | N/A (reference) | Mutable pointer |
| `RawPtr<T>` | `T*` | N/A | N/A | Unsafe raw pointer |

JS and Python don't have raw pointers - references are used instead. For FFI, use typed arrays (`Int32Array`, etc.).

### 4.5 Algebraic Types

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `Option<T>` | Tagged union | `T \| null` | `Optional[T]` | Some/None |
| `Result<T, E>` | Tagged union | `{ok: T} \| {err: E}` | Result class | Ok/Err |

**C encoding for Option<T>:**
```c
typedef struct {
    bool is_some;
    T value;  // Only valid if is_some == true
} zen_option_t;
```

**C encoding for Result<T, E>:**
```c
typedef struct {
    bool is_ok;
    union {
        T ok_value;
        E err_value;
    };
} zen_result_t;
```

**JS encoding for Result:**
```javascript
// Result as discriminated union
type Result<T, E> = { ok: true, value: T } | { ok: false, error: E };
```

**Python encoding for Result:**
```python
from typing import Generic, TypeVar, Union

T = TypeVar('T')
E = TypeVar('E')

class Ok(Generic[T]):
    def __init__(self, value: T):
        self.value = value

class Err(Generic[E]):
    def __init__(self, error: E):
        self.error = error

Result = Union[Ok[T], Err[E]]
```

### 4.6 Structs

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `Point: { x: i32, y: i32 }` | `struct { int32_t x; int32_t y; }` | `{x: number, y: number}` | `@dataclass class Point` | Product type |

**C:**
```c
typedef struct {
    int32_t x;
    int32_t y;
} zen_point_t;
```

**JS:**
```javascript
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
}
```

**Python:**
```python
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int
```

### 4.7 Enums (Sum Types)

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `Color: Red, Green, Blue` | `enum { RED, GREEN, BLUE }` | Numeric enum or symbol | `Enum` subclass | Simple enum |
| `Option<T>: Some(T), None` | Tagged union | Discriminated union | Enum with methods | Enum with payload |

**C encoding for enums with payloads:**
```c
typedef enum {
    ZEN_OPTION_SOME,
    ZEN_OPTION_NONE
} zen_option_tag_t;

typedef struct {
    zen_option_tag_t tag;
    union {
        int32_t some_value;  // If tag == SOME
    };
} zen_option_i32_t;
```

**JS:**
```javascript
class Option {
    static Some(value) { return { tag: 'Some', value }; }
    static None() { return { tag: 'None' }; }
}
```

**Python:**
```python
from enum import Enum
from typing import Generic, TypeVar

T = TypeVar('T')

class Option(Generic[T], Enum):
    def Some(value: T):
        return ('Some', value)

    def None():
        return ('None', None)
```

### 4.8 Functions and Closures

| Zen Type | C Type | JS Type | Python Type | Notes |
|----------|--------|---------|-------------|-------|
| `(i32, i32) i32` | `int32_t (*)(int32_t, int32_t)` | `(a: number, b: number) => number` | `Callable[[int, int], int]` | Function pointer |
| Closure | `struct { fn_ptr, env* }` | Arrow function | Lambda | Captures environment |

**C closure encoding:**
```c
typedef struct {
    int32_t (*fn)(void* env, int32_t, int32_t);
    void* env;  // Captured variables
} zen_closure_t;
```

### 4.9 Pattern Matching

| Zen Construct | C Translation | JS Translation | Python Translation |
|---------------|---------------|----------------|-------------------|
| `expr ? \| Pattern { body }` | `switch` or `if/else` chain | `switch` or `if/else` | `match` (3.10+) or `if/elif` |
| Range pattern `1..=10` | `if (x >= 1 && x <= 10)` | `if (x >= 1 && x <= 10)` | `if 1 <= x <= 10:` |
| Struct destructuring | Field access | Destructuring | Dataclass pattern |

**Example: Zen → C**
```zen
result ?
    | Ok(val) { return val }
    | Err(e) { log(e); return 0 }
```

```c
if (result.is_ok) {
    return result.ok_value;
} else {
    log(result.err_value);
    return 0;
}
```

---

## 5. The Self-Hosting Path

### Phase 1: Meta Module Exposes AST (Rust implements intrinsics)

**What's built:**
- `ComptimeValue::ASTNode(Rc<ASTNode>)` variant in `/home/ubuntu/zenlang/src/comptime/mod.rs`
- `meta.type_info(ast_node)` returns `TypeInfo` with AST structure
- `meta.fields(ast_node)` gives field access
- `meta.variant_name(ast_node)` returns enum variant name
- Parser produces `ComptimeValue::ASTNode` values

**Example:**
```zen
{ meta } = @std

// This works now:
program = parse("main = () void { @std.io.println(\"Hello\") }")
program.declarations.loop((decl) {
    info = meta.type_info(decl)
    @std.io.println("Declaration kind: ${info.kind}")
})
```

**Implementation in Rust:**
```rust
// In src/comptime/mod.rs
pub enum ComptimeValue {
    // ... existing variants
    ASTNode(Rc<ASTNodeValue>),
}

pub enum ASTNodeValue {
    Expression(ast::Expression),
    Statement(ast::Statement),
    Declaration(ast::Declaration),
    Type(ast::AstType),
    Pattern(ast::Pattern),
}

impl ComptimeInterpreter {
    pub fn expose_ast_node(&mut self, node: ASTNodeValue) -> ComptimeValue {
        ComptimeValue::ASTNode(Rc::new(node))
    }
}
```

### Phase 2: Code Generators Written in Zen

**What's built:**
- `stdlib/codegen/c_emitter.zen` - C backend
- `stdlib/codegen/js_emitter.zen` - JavaScript backend
- `stdlib/codegen/python_emitter.zen` - Python backend
- `stdlib/codegen/walker.zen` - Generic AST walker
- `stdlib/codegen/emitter.zen` - Emitter behavior interface

**Example C emitter (simplified):**
```zen
// stdlib/codegen/c_emitter.zen
{ meta, io, string } = @std

CEmitter: {
    buffer:: string.StringBuilder,
    indent:: u32,
    forward_decls:: Vec<String>,
    in_function:: bool,
}

CEmitter.new = () CEmitter {
    return CEmitter {
        buffer: string.StringBuilder.new(),
        indent: 0,
        forward_decls: Vec.new(),
        in_function: false,
    }
}

CEmitter.implements(Emitter, {
    emit_function_start = (self, name, args, return_type) void {
        // Collect forward declaration
        decl = type_to_c(return_type) + " " + name + "("
        args.loop((arg, i) {
            i > 0 ? { decl = decl + ", " }
            decl = decl + type_to_c(arg.1) + " " + arg.0
        })
        decl = decl + ");"
        self.forward_decls.push(decl)

        // Emit function definition
        self.emit_indent()
        self.buffer.append(type_to_c(return_type) + " " + name + "(")
        args.loop((arg, i) {
            i > 0 ? { self.buffer.append(", ") }
            self.buffer.append(type_to_c(arg.1) + " " + arg.0)
        })
        self.buffer.append(") {\n")
        self.indent = self.indent + 1
        self.in_function = true
    },

    emit_function_end = (self) void {
        self.indent = self.indent - 1
        self.emit_indent()
        self.buffer.append("}\n\n")
        self.in_function = false
    },

    emit_int = (self, val) void {
        self.buffer.append(val.to_string())
    },

    emit_string = (self, val) void {
        self.buffer.append("\"" + escape_c_string(val) + "\"")
    },

    emit_operator = (self, op) void {
        op_str = op ?
            | Add { "+" }
            | Subtract { "-" }
            | Multiply { "*" }
            | Divide { "/" }
            | Modulo { "%" }
            | Equals { "==" }
            | NotEquals { "!=" }
            | LessThan { "<" }
            | GreaterThan { ">" }
            | LessThanEquals { "<=" }
            | GreaterThanEquals { ">=" }
            | And { "&&" }
            | Or { "||" }
            | BitwiseAnd { "&" }
            | BitwiseOr { "|" }
            | BitwiseXor { "^" }
            | ShiftLeft { "<<" }
            | ShiftRight { ">>" }
        self.buffer.append(" " + op_str + " ")
    },

    output = (self) String {
        // Prepend forward declarations
        result = "#include <stdint.h>\n#include <stdbool.h>\n\n"
        self.forward_decls.loop((decl) {
            result = result + decl + "\n"
        })
        result = result + "\n" + self.buffer.to_string()
        return result
    },

    // ... other emitter methods
})

type_to_c = (type_: AstType) String {
    type_ ?
        | I8 { "int8_t" }
        | I16 { "int16_t" }
        | I32 { "int32_t" }
        | I64 { "int64_t" }
        | U8 { "uint8_t" }
        | U16 { "uint16_t" }
        | U32 { "uint32_t" }
        | U64 { "uint64_t" }
        | Usize { "size_t" }
        | F32 { "float" }
        | F64 { "double" }
        | Bool { "bool" }
        | Void { "void" }
        | Struct(s) { "zen_" + s.name + "_t" }
        | Generic(g) {
            // Handle Ptr<T>, Option<T>, etc.
            g.name == "Ptr" ? { type_to_c(g.type_args[0]) + "*" }
            g.name == "MutPtr" ? { type_to_c(g.type_args[0]) + "*" }
            g.name == "Option" ? { "zen_option_" + type_to_c(g.type_args[0]) + "_t" }
            { "zen_" + g.name + "_t" }  // Fallback
        }
        | _ { "void*" }  // Unknown types become void*
}
```

**Invocation:**
```zen
// User code
{ codegen } = @std
{ CEmitter, walk_program } = codegen

program = parse_file("my_program.zen")
emitter = CEmitter.new()
walk_program(program, emitter)
c_code = emitter.output()
io.write_file("output.c", c_code)
```

**Critical insight:** This C emitter is **not part of the compiler**. It's a user-level Zen library. The compiler just needs to expose the AST through `meta`.

### Phase 3: Optimization Passes in Zen

**What's built:**
- `stdlib/optimizer/constant_folding.zen`
- `stdlib/optimizer/dead_code_elimination.zen`
- `stdlib/optimizer/inline_expansion.zen`

**Example: Constant folding**
```zen
// stdlib/optimizer/constant_folding.zen
{ meta } = @std

optimize_expression = (expr: Expression) Expression {
    expr ?
        | BinaryOp(op) {
            left = optimize_expression(op.left.val)
            right = optimize_expression(op.right.val)

            // Fold constant operations
            (left, right) ?
                | (Integer32(l), Integer32(r)) {
                    op.op ?
                        | Add { return .Integer32(l + r) }
                        | Subtract { return .Integer32(l - r) }
                        | Multiply { return .Integer32(l * r) }
                        | Divide { r != 0 ? { return .Integer32(l / r) } }
                        | _ { }
                }
                | _ { }

            // Return optimized BinaryOp
            return .BinaryOp({
                left: @std.heap.alloc(left),
                op: op.op,
                right: @std.heap.alloc(right),
            })
        }
        | FunctionCall(call) {
            // Optimize arguments
            optimized_args = call.args.map((arg) { optimize_expression(arg) })
            return .FunctionCall({
                name: call.name,
                type_args: call.type_args,
                args: optimized_args,
            })
        }
        | _ { return expr }  // No optimization
}
```

**Usage:**
```zen
program = parse("x = 2 + 3; y = x * 4")
optimized = optimize_program(program)
// Now "x = 5; y = x * 4"
```

### Phase 4: Parser in Zen (Speculative)

This is ambitious but theoretically possible:

- `stdlib/parser/lexer.zen` - Tokenization
- `stdlib/parser/parser.zen` - AST construction
- `stdlib/parser/combinator.zen` - Parser combinators

The parser would **return AST nodes** as `ComptimeValue::ASTNode` values. This closes the loop: Zen parses Zen, producing Zen-visible AST, which Zen code generators consume.

**Chicken-and-egg solution:** Bootstrap with the Rust parser, then switch to the Zen parser once it's working.

### Phase 5: Full Self-Hosting

**The compiler is now:**
```zen
// compiler.zen
{ io, codegen, optimizer, parser } = @std

compile = (source_file: String, output_file: String) Result<void, String> {
    // Parse
    source = io.read_file(source_file).raise()
    program = parser.parse(source).raise()

    // Optimize
    optimized = optimizer.optimize(program)

    // Generate C
    c_code = codegen.to_c(optimized)
    io.write_file(output_file, c_code).raise()

    // Compile C to binary
    @std.process.run("gcc", ["-o", "a.out", output_file]).raise()

    return .Ok(())
}
```

**Bootstrapping:**
1. Compile `compiler.zen` with the Rust compiler → `zen-compiler` (native binary)
2. Use `zen-compiler` to compile itself → `zen-compiler2`
3. Verify `zen-compiler` and `zen-compiler2` are identical (reproducible build)
4. Ship `zen-compiler2` as the official compiler

**The Rust compiler remains** as a fallback and for LLVM-based native compilation. But the C backend path is 100% Zen code.

---

## 6. State Machine Architecture

### 6.1 Emitter State Components

Every emitter needs mutable state to track context during traversal:

```zen
EmitterState: {
    // Output buffer
    buffer:: StringBuilder,

    // Formatting state
    indent_level:: u32,
    indent_string:: String,  // "    " or "\t"
    newline_pending:: bool,

    // Scope tracking
    scope_depth:: u32,
    current_scope:: Scope,
    scope_stack:: Vec<Scope>,

    // Context flags (state machine)
    in_function:: bool,
    in_loop:: bool,
    in_match_arm:: bool,
    in_closure:: bool,
    in_unsafe_block:: bool,

    // Function context
    current_function_name:: Option<String>,
    current_function_return_type:: Option<AstType>,
    has_return_statement:: bool,

    // Loop context (for break/continue)
    current_loop_label:: Option<String>,
    loop_depth:: u32,

    // Type context
    current_type_name:: Option<String>,  // When inside a struct/enum
    generic_bindings:: HashMap<String, AstType>,  // T -> i32 substitutions
}

Scope: {
    variables:: HashMap<String, VariableInfo>,
    depth:: u32,
}

VariableInfo: {
    name:: String,
    type_:: AstType,
    is_mutable:: bool,
    is_used:: bool,  // For unused variable warnings
}
```

### 6.2 C-Specific State

C requires forward declarations and header/source split:

```zen
CEmitterState: {
    // Inherits EmitterState
    base:: EmitterState,

    // Forward declarations
    forward_decls:: Vec<String>,
    struct_forward_decls:: Vec<String>,

    // Dependency tracking
    used_stdlib_functions:: HashSet<String>,
    required_includes:: HashSet<String>,

    // Name mangling (for generics)
    mangled_names:: HashMap<String, String>,  // Vec<i32> -> zen_vec_i32_t

    // Memory management
    needs_free_calls:: Vec<String>,  // Variables that need cleanup

    // Enum/union tracking
    defined_enums:: HashSet<String>,
    defined_unions:: HashSet<String>,
}

// C emitter tracks which stdlib functions are used
track_stdlib_usage = (state: CEmitterState, func_name: String) void {
    state.used_stdlib_functions.insert(func_name)

    // Add required includes
    func_name ?
        | "malloc" | "free" | "realloc" {
            state.required_includes.insert("<stdlib.h>")
        }
        | "printf" | "fprintf" | "sprintf" {
            state.required_includes.insert("<stdio.h>")
        }
        | "strlen" | "strcpy" | "strcat" {
            state.required_includes.insert("<string.h>")
        }
        | _ { }
}
```

### 6.3 JavaScript-Specific State

JavaScript needs hoisting tracking and module exports:

```zen
JSEmitterState: {
    base:: EmitterState,

    // Hoisting (function declarations vs. expressions)
    hoisted_functions:: Vec<String>,

    // Module system
    export_list:: Vec<String>,
    import_statements:: Vec<String>,

    // Async context
    in_async_function:: bool,
    needs_await:: bool,

    // Closure scope chain
    closure_variables:: Vec<HashSet<String>>,

    // Strict mode
    use_strict:: bool,
}

// JS emitter handles async/await transformation
emit_async_call = (state: JSEmitterState, call: FunctionCall) void {
    state.in_async_function ? {
        state.buffer.append("await ")
    }
    emit_call(state, call)
}
```

### 6.4 Python-Specific State

Python requires indentation tracking and type hints:

```zen
PythonEmitterState: {
    base:: EmitterState,

    // Indentation (Python is whitespace-sensitive)
    indent_level:: u32,  // Number of spaces (4 per level)

    // Type hints
    use_type_hints:: bool,
    imported_types:: HashSet<String>,  // typing.Optional, etc.

    // Import tracking
    import_statements:: Vec<String>,
    from_imports:: HashMap<String, Vec<String>>,  // "typing" -> ["Optional", "List"]

    // Decorator tracking
    pending_decorators:: Vec<String>,

    // Class context
    in_class:: bool,
    current_class_name:: Option<String>,
}

// Python emitter generates type hints
emit_type_hint = (state: PythonEmitterState, type_: AstType) void {
    state.use_type_hints ? {
        hint = type_ ?
            | I32 { "int" }
            | F64 { "float" }
            | Bool { "bool" }
            | Generic(g) {
                g.name == "Option" ? {
                    state.from_imports.entry("typing").push("Optional")
                    "Optional[" + type_hint(g.type_args[0]) + "]"
                }
                g.name == "Vec" ? {
                    state.from_imports.entry("typing").push("List")
                    "List[" + type_hint(g.type_args[0]) + "]"
                }
                { g.name }
            }
            | _ { "Any" }
        state.buffer.append(": " + hint)
    }
}
```

### 6.5 State Transitions (Example: Function Compilation)

```zen
emit_function = (state: EmitterState, func: Function) void {
    // Save previous state
    prev_in_function = state.in_function
    prev_function_name = state.current_function_name
    prev_return_type = state.current_function_return_type
    prev_has_return = state.has_return_statement

    // Enter function context
    state.in_function = true
    state.current_function_name = .Some(func.name)
    state.current_function_return_type = .Some(func.return_type)
    state.has_return_statement = false

    // Create new scope for function parameters
    push_scope(state)
    func.args.loop((arg) {
        define_variable(state, arg.0, arg.1, false)  // Parameters are immutable by default
    })

    // Emit function signature
    emit_function_signature(state, func)

    // Emit function body
    emit_block_start(state)
    func.body.loop((stmt) {
        emit_statement(state, stmt)
    })
    emit_block_end(state)

    // Check if void function needs implicit return
    func.return_type == .Void and not state.has_return_statement ? {
        emit_implicit_return(state)
    }

    // Restore previous state
    pop_scope(state)
    state.in_function = prev_in_function
    state.current_function_name = prev_function_name
    state.current_function_return_type = prev_return_type
    state.has_return_statement = prev_has_return
}
```

### 6.6 Scope Management

```zen
Scope: {
    variables:: HashMap<String, VariableInfo>,
    depth:: u32,
    parent:: Option<Ptr<Scope>>,
}

push_scope = (state: EmitterState) void {
    new_scope = Scope {
        variables: HashMap.new(),
        depth: state.scope_depth + 1,
        parent: .Some(@std.ptr.ref(state.current_scope)),
    }
    state.scope_stack.push(state.current_scope)
    state.current_scope = new_scope
    state.scope_depth = state.scope_depth + 1
}

pop_scope = (state: EmitterState) void {
    state.current_scope = state.scope_stack.pop().unwrap()
    state.scope_depth = state.scope_depth - 1
}

define_variable = (state: EmitterState, name: String, type_: AstType, is_mutable: bool) void {
    info = VariableInfo {
        name: name,
        type_: type_,
        is_mutable: is_mutable,
        is_used: false,
    }
    state.current_scope.variables.insert(name, info)
}

lookup_variable = (state: EmitterState, name: String) Option<VariableInfo> {
    // Search current scope
    state.current_scope.variables.get(name) ? | Some(info) { return .Some(info) }

    // Search parent scopes
    current = state.current_scope.parent
    loop {
        current ? | Some(scope) {
            scope.variables.get(name) ? | Some(info) { return .Some(info) }
            current = scope.parent
        } | None {
            break
        }
    }

    return .None
}
```

---

## 7. Advantages Over Hardcoded Rust Backends

### 7.1 One Implementation, Unlimited Targets

**Current (Rust-based) approach:**
- LLVM backend: ~5000 lines of Rust in `/home/ubuntu/zenlang/src/codegen/llvm/`
- Each new backend (C, JS, Python, WASM) requires:
  - ~3000-5000 lines of Rust
  - Deep compiler knowledge
  - Rust proficiency
  - Integration with existing compiler phases

**Meta-based approach:**
- Rust implements `meta.type_info()` **once** (~500 lines)
- Each backend is ~1000-2000 lines of **Zen code**
- No compiler knowledge required - just target language knowledge
- Backends are testable as standalone Zen programs

### 7.2 User Extensibility

**Impossible in Rust-based compiler:**
- User wants to generate custom DSL output
- User wants to generate documentation from AST
- User wants to generate GraphQL schema from types
- **Solution:** User must fork the compiler, add Rust module, rebuild

**Trivial with meta-based approach:**
```zen
// User code in my_project/
{ meta, codegen } = @std

MyDSLEmitter: { buffer:: StringBuilder }
MyDSLEmitter.implements(Emitter, { /* custom emit methods */ })

to_my_dsl = (program: Program) String {
    emitter = MyDSLEmitter.new()
    codegen.walk_program(program, emitter)
    return emitter.output()
}

// Usage:
my_program = parse("...")
dsl_output = to_my_dsl(my_program)
```

No compiler modification needed. The user just imports `@std.meta` and writes Zen code.

### 7.3 Composable Transformations

**Rust approach:** Optimization passes are hardcoded in the compiler pipeline. Adding a new pass requires:
- Modifying compiler internals
- Understanding the entire compilation flow
- Rebuilding the compiler

**Meta approach:** Optimization passes are functions that transform AST:
```zen
// User writes a custom optimization pass
inline_small_functions = (program: Program) Program {
    // Walk AST, find small functions, inline them
    // Return transformed AST
}

// Compose with existing passes
optimize = (program: Program) Program {
    program
        .pipe(constant_folding)
        .pipe(dead_code_elimination)
        .pipe(inline_small_functions)  // User's custom pass!
        .pipe(common_subexpression_elimination)
}
```

### 7.4 Testability

**Rust backend testing:**
- Requires running the entire compiler
- Input: Zen source code
- Output: LLVM IR or binary
- Hard to test edge cases in isolation

**Meta backend testing:**
```zen
// Test a single emit function
test_emit_binary_op = () void {
    expr = .BinaryOp({
        left: @std.heap.alloc(.Integer32(2)),
        op: .Add,
        right: @std.heap.alloc(.Integer32(3)),
    })

    emitter = CEmitter.new()
    walk_expression(expr, emitter)

    expected = "2 + 3"
    actual = emitter.buffer.to_string()
    assert(actual == expected, "Binary op emission failed")
}
```

Each emit function is a pure function that can be tested in isolation.

### 7.5 Debugging and Iteration Speed

**Rust backend development:**
- Change codegen logic
- Rebuild entire compiler (20-60 seconds)
- Run test
- Repeat

**Meta backend development:**
- Change emit function in `stdlib/codegen/c_emitter.zen`
- Run `zen test` (2-5 seconds - no compiler rebuild!)
- Iterate

### 7.6 Community Contributions

**Rust approach:**
- Contributing a new backend requires:
  - Rust expertise
  - Compiler internals knowledge
  - LLVM or code generation experience
  - Weeks of work

**Meta approach:**
- Contributing a new backend requires:
  - Zen knowledge (which users already have)
  - Target language knowledge
  - ~1-2 days of work

**Result:** Community can easily add backends for:
- WebAssembly
- JVM bytecode
- ARM assembly
- Custom hardware DSLs
- Domain-specific languages
- Documentation formats (LaTeX, Markdown)

### 7.7 Same Meta Module Enables Other Features

The `meta` module that exposes AST also enables:

**Serialization:**
```zen
serialize_to_json = (value: T) String {
    info = meta.type_info(value)
    info.kind ?
        | Struct(s) {
            json = "{"
            s.fields.loop((field, i) {
                i > 0 ? { json = json + ", " }
                json = json + "\"" + field.name + "\": " + serialize_to_json(field.value)
            })
            json + "}"
        }
        | _ { value.to_string() }
}
```

**Documentation generation:**
```zen
generate_docs = (program: Program) String {
    docs = ""
    program.declarations.loop((decl) {
        decl ? | Function(f) {
            docs = docs + "## " + f.name + "\n"
            docs = docs + "**Arguments:** " + f.args.map(arg_to_string).join(", ") + "\n"
            docs = docs + "**Returns:** " + type_to_string(f.return_type) + "\n\n"
        }
    })
    return docs
}
```

**Testing frameworks:**
```zen
find_test_functions = (program: Program) []Function {
    tests = Vec.new()
    program.declarations.loop((decl) {
        decl ? | Function(f) {
            f.name.starts_with("test_") ? {
                tests.push(f)
            }
        }
    })
    return tests
}
```

---

## 8. What Needs to Be Implemented (In Rust)

### 8.1 Extend ComptimeValue

In `/home/ubuntu/zenlang/src/comptime/mod.rs`:

```rust
#[derive(Debug, Clone)]
pub enum ComptimeValue {
    // ... existing variants (I32, Bool, String, etc.)

    // NEW: AST node references
    ASTNode(Rc<ASTNodeValue>),
}

#[derive(Debug, Clone)]
pub enum ASTNodeValue {
    Expression(ast::Expression),
    Statement(ast::Statement),
    Declaration(ast::Declaration),
    Type(ast::AstType),
    Pattern(ast::Pattern),
}

impl ComptimeValue {
    pub fn to_expression(&self) -> Result<Expression> {
        match self {
            // ... existing conversions
            ComptimeValue::ASTNode(node) => {
                match node.as_ref() {
                    ASTNodeValue::Expression(e) => Ok(e.clone()),
                    _ => Err(CompileError::ComptimeError(
                        "Cannot convert non-expression AST node to expression".to_string(),
                        None,
                    )),
                }
            }
        }
    }
}
```

### 8.2 Implement meta.type_info() for AST Nodes

In a new file `/home/ubuntu/zenlang/src/comptime/meta_introspection.rs`:

```rust
use crate::ast::*;
use crate::comptime::{ComptimeValue, ComptimeInterpreter};
use crate::error::Result;
use std::collections::HashMap;

impl ComptimeInterpreter {
    /// meta.type_info(ast_node) -> TypeInfo
    pub fn meta_type_info(&self, node: ComptimeValue) -> Result<ComptimeValue> {
        match node {
            ComptimeValue::ASTNode(ast_node) => {
                let type_info = match ast_node.as_ref() {
                    ASTNodeValue::Expression(expr) => self.expression_type_info(expr)?,
                    ASTNodeValue::Statement(stmt) => self.statement_type_info(stmt)?,
                    ASTNodeValue::Declaration(decl) => self.declaration_type_info(decl)?,
                    ASTNodeValue::Type(ty) => self.type_type_info(ty)?,
                    ASTNodeValue::Pattern(pat) => self.pattern_type_info(pat)?,
                };
                Ok(type_info)
            }
            _ => Err(CompileError::ComptimeError(
                "meta.type_info expects an AST node".to_string(),
                None,
            )),
        }
    }

    fn expression_type_info(&self, expr: &Expression) -> Result<ComptimeValue> {
        let (name, fields) = match expr {
            Expression::Integer32(val) => {
                ("Integer32".to_string(), vec![("value".to_string(), ComptimeValue::I32(*val))])
            }
            Expression::BinaryOp { left, op, right } => {
                ("BinaryOp".to_string(), vec![
                    ("left".to_string(), ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(*left.clone())))),
                    ("op".to_string(), self.binary_op_to_comptime(op)),
                    ("right".to_string(), ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(*right.clone())))),
                ])
            }
            Expression::FunctionCall { name, type_args, args } => {
                ("FunctionCall".to_string(), vec![
                    ("name".to_string(), ComptimeValue::String(name.clone())),
                    ("type_args".to_string(), self.type_args_to_comptime(type_args)),
                    ("args".to_string(), self.args_to_comptime(args)),
                ])
            }
            // ... handle all Expression variants
            _ => return Err(CompileError::ComptimeError(
                format!("Unsupported expression variant: {:?}", expr),
                None,
            )),
        };

        Ok(ComptimeValue::Struct {
            name: "TypeInfo".to_string(),
            fields: HashMap::from([
                ("name".to_string(), ComptimeValue::String(name)),
                ("kind".to_string(), ComptimeValue::Struct {
                    name: "TypeKind::Struct".to_string(),
                    fields: HashMap::from([
                        ("fields".to_string(), ComptimeValue::Array(fields.into_iter().map(|(k, v)| {
                            ComptimeValue::Struct {
                                name: "FieldInfo".to_string(),
                                fields: HashMap::from([
                                    ("name".to_string(), ComptimeValue::String(k)),
                                    ("value".to_string(), v),
                                ]),
                            }
                        }).collect())),
                    ]),
                }),
            ]),
        })
    }

    // Similar implementations for statement_type_info, declaration_type_info, etc.
}
```

### 8.3 Implement meta.fields()

```rust
impl ComptimeInterpreter {
    /// meta.fields(ast_node) -> []FieldInfo
    pub fn meta_fields(&self, node: ComptimeValue) -> Result<ComptimeValue> {
        match node {
            ComptimeValue::ASTNode(ast_node) => {
                let fields = match ast_node.as_ref() {
                    ASTNodeValue::Expression(expr) => self.expression_fields(expr)?,
                    ASTNodeValue::Statement(stmt) => self.statement_fields(stmt)?,
                    ASTNodeValue::Declaration(decl) => self.declaration_fields(decl)?,
                    ASTNodeValue::Type(ty) => self.type_fields(ty)?,
                    ASTNodeValue::Pattern(pat) => self.pattern_fields(pat)?,
                };
                Ok(ComptimeValue::Array(fields))
            }
            _ => Err(CompileError::ComptimeError(
                "meta.fields expects an AST node".to_string(),
                None,
            )),
        }
    }

    fn expression_fields(&self, expr: &Expression) -> Result<Vec<ComptimeValue>> {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                Ok(vec![
                    self.make_field_info("left", ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(*left.clone())))),
                    self.make_field_info("op", self.binary_op_to_comptime(op)),
                    self.make_field_info("right", ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(*right.clone())))),
                ])
            }
            // ... handle other variants
            _ => Ok(vec![]),
        }
    }

    fn make_field_info(&self, name: &str, value: ComptimeValue) -> ComptimeValue {
        ComptimeValue::Struct {
            name: "FieldInfo".to_string(),
            fields: HashMap::from([
                ("name".to_string(), ComptimeValue::String(name.to_string())),
                ("value".to_string(), value),
            ]),
        }
    }
}
```

### 8.4 Implement meta.variant_name()

```rust
impl ComptimeInterpreter {
    /// meta.variant_name(enum_value) -> String
    pub fn meta_variant_name(&self, node: ComptimeValue) -> Result<ComptimeValue> {
        match node {
            ComptimeValue::ASTNode(ast_node) => {
                let variant = match ast_node.as_ref() {
                    ASTNodeValue::Expression(expr) => expression_variant_name(expr),
                    ASTNodeValue::Statement(stmt) => statement_variant_name(stmt),
                    ASTNodeValue::Declaration(decl) => declaration_variant_name(decl),
                    ASTNodeValue::Type(ty) => type_variant_name(ty),
                    ASTNodeValue::Pattern(pat) => pattern_variant_name(pat),
                };
                Ok(ComptimeValue::String(variant))
            }
            _ => Err(CompileError::ComptimeError(
                "meta.variant_name expects an AST node".to_string(),
                None,
            )),
        }
    }
}

fn expression_variant_name(expr: &Expression) -> String {
    match expr {
        Expression::Integer32(_) => "Integer32".to_string(),
        Expression::BinaryOp { .. } => "BinaryOp".to_string(),
        Expression::FunctionCall { .. } => "FunctionCall".to_string(),
        // ... all variants
        _ => "Unknown".to_string(),
    }
}
```

### 8.5 Expose AST in Parser Output

In `/home/ubuntu/zenlang/src/parser/core.rs`:

```rust
impl Parser {
    pub fn parse_to_comptime(&mut self) -> Result<ComptimeValue> {
        let program = self.parse()?;
        Ok(ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
            ast::Declaration::Program(program)
        ))))
    }
}
```

### 8.6 Add meta Intrinsics to Comptime Function Registry

In `/home/ubuntu/zenlang/src/comptime/mod.rs`:

```rust
impl ComptimeInterpreter {
    fn init_builtins(&mut self) {
        // ... existing builtins

        // Meta introspection functions
        self.register_intrinsic("meta.type_info", |interp, args| {
            if args.len() != 1 {
                return Err(CompileError::ComptimeError(
                    "meta.type_info expects 1 argument".to_string(),
                    None,
                ));
            }
            interp.meta_type_info(args[0].clone())
        });

        self.register_intrinsic("meta.fields", |interp, args| {
            if args.len() != 1 {
                return Err(CompileError::ComptimeError(
                    "meta.fields expects 1 argument".to_string(),
                    None,
                ));
            }
            interp.meta_fields(args[0].clone())
        });

        self.register_intrinsic("meta.variant_name", |interp, args| {
            if args.len() != 1 {
                return Err(CompileError::ComptimeError(
                    "meta.variant_name expects 1 argument".to_string(),
                    None,
                ));
            }
            interp.meta_variant_name(args[0].clone())
        });
    }
}
```

### 8.7 Bootstrap: Keep LLVM Backend, Add C Backend in Zen

**Critical insight:** We **don't replace** the LLVM backend. It remains for native compilation. The C backend is **additional**, written in Zen, for bootstrapping.

**Compilation modes:**
1. **Native mode:** Zen → LLVM IR → native binary (fast, current approach)
2. **Bootstrap mode:** Zen → C code (via Zen emitter) → gcc → binary (self-hosting)
3. **Development mode:** Zen → JS/Python (via Zen emitter) → interpreter (fast iteration)

All three coexist. Users choose via compiler flags:
```bash
zen compile --target=native program.zen    # LLVM backend (Rust)
zen compile --target=c program.zen         # C backend (Zen)
zen compile --target=js program.zen        # JS backend (Zen)
```

---

## 9. Risk Assessment

### Risk 1: Complexity of Exposing Full AST Through Comptime

**Risk:** The AST has 40+ expression variants, 13 statement variants, 12 declaration variants. Exposing all of this through `meta.type_info()` is a large implementation surface.

**Severity:** Medium

**Mitigation:**
- **Incremental implementation:** Start with a subset (10-15 most common expression variants)
- **Code generation:** Use Rust macros to auto-generate boilerplate for each variant
- **Testing:** Comprehensive tests for each variant's introspection
- **Documentation:** Clear mapping from Rust AST types to Zen-visible types

**Fallback:** If full AST exposure proves too complex, expose a **simplified AST** with fewer variants, and expand gradually.

### Risk 2: Performance - Comptime Interpretation vs Native Rust Codegen

**Risk:** Zen code (interpreted at compile time) will be slower than native Rust code. Code generation could become a bottleneck.

**Severity:** Medium

**Analysis:**
- **Current LLVM codegen speed:** ~1000-5000 LOC/second (baseline)
- **Estimated Zen emitter speed:** ~100-500 LOC/second (10x slower)
- **Impact:** For a 10,000 line program, codegen time increases from 2s to 20s

**Mitigation:**
- **JIT compilation of Zen emitters:** Compile Zen emitters to native code before running them
- **Caching:** Cache codegen results for unchanged modules
- **Parallelization:** Run codegen for multiple modules in parallel
- **Hybrid approach:** Use Rust LLVM backend for production, Zen emitters for development/bootstrapping

**Reality check:** Zig's comptime is interpreted and still practical. Rust's procedural macros are interpreted and widely used. 10x slowdown in codegen is acceptable if compilation is still fast overall.

### Risk 3: Bootstrapping Chicken-and-Egg

**Risk:** To compile the Zen C emitter, we need a working Zen compiler. But we're trying to replace the Rust compiler with the Zen C emitter.

**Severity:** Low (solvable)

**Solution:**
1. **Phase 1:** Rust compiler compiles Zen C emitter → `c_emitter.o` (native)
2. **Phase 2:** Rust compiler uses `c_emitter.o` to compile Zen programs to C
3. **Phase 3:** `c_emitter.o` compiles itself → `c_emitter2.o`
4. **Phase 4:** Verify `c_emitter.o` and `c_emitter2.o` produce identical output (reproducible build)
5. **Ship:** `c_emitter2.o` becomes the official C backend

This is standard bootstrapping - no fundamental blocker.

### Risk 4: Incomplete Meta Coverage

**Risk:** Some AST patterns may be difficult or impossible to express through `meta.type_info()`.

**Examples:**
- Recursive types (AST nodes contain pointers to other AST nodes)
- Large enums with 40+ variants (pattern matching on all variants is tedious)
- Span information (source locations) - should this be exposed?

**Severity:** Medium

**Mitigation:**
- **Helper functions:** Provide high-level helpers in `stdlib/codegen/walker.zen` that abstract over common patterns
- **Pattern matching sugar:** Use `?` operator with `_` wildcard to handle "uninteresting" variants
- **Selective exposure:** Don't expose **everything** - only what's needed for codegen
- **Escape hatch:** If meta is insufficient, provide a Rust FFI function to access specific data

**Example helper:**
```zen
// Helper: Visit only "interesting" expressions
visit_expression = (expr: Expression, visitor: ExpressionVisitor) void {
    expr ?
        | BinaryOp(op) { visitor.visit_binary_op(op) }
        | FunctionCall(call) { visitor.visit_call(call) }
        | QuestionMatch(match_) { visitor.visit_match(match_) }
        | _ { visitor.visit_other(expr) }  // Catch-all for simple cases
}
```

### Risk 5: Type Safety Across Boundaries

**Risk:** Zen emitters produce strings (C code, JS code). There's no compile-time verification that the emitted code is valid in the target language.

**Severity:** Medium

**Mitigation:**
- **Golden tests:** Extensive test suite with known-good inputs and outputs
- **Round-trip testing:** Emit C, compile with gcc, run, verify output matches Zen program
- **Type-directed emission:** Use target language's type system to guide emission
- **Linting:** Run target language's linter on emitted code (clang-tidy, eslint, mypy)

**Reality:** This is the same challenge as any code generator (LLVM IR, assembly). The solution is thorough testing.

---

## 10. Concrete Next Steps

### Step 1: Extend ComptimeValue to Hold AST Nodes

**Implementation:**
- Add `ComptimeValue::ASTNode(Rc<ASTNodeValue>)` variant
- Add `ASTNodeValue` enum with `Expression`, `Statement`, `Declaration`, `Type`, `Pattern` variants
- Implement `ComptimeValue::to_expression()` for AST nodes
- Write tests for AST node creation and conversion

**Test:**
```rust
#[test]
fn test_ast_node_comptime_value() {
    let expr = Expression::Integer32(42);
    let comptime_val = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(expr.clone())));

    match comptime_val {
        ComptimeValue::ASTNode(node) => {
            match node.as_ref() {
                ASTNodeValue::Expression(e) => assert_eq!(e, &expr),
                _ => panic!("Wrong AST node type"),
            }
        }
        _ => panic!("Not an AST node"),
    }
}
```

### Step 2: Implement meta.type_info() for a Subset of Expressions

**Start with these 10 variants:**
1. `Integer32`
2. `String`
3. `Identifier`
4. `BinaryOp`
5. `FunctionCall`
6. `Block`
7. `QuestionMatch`
8. `MethodCall`
9. `Return`
10. `Break`

**Implementation:**
- Create `/home/ubuntu/zenlang/src/comptime/meta_introspection.rs`
- Implement `meta_type_info()` for these 10 variants
- Register `meta.type_info` as a comptime intrinsic
- Write tests for each variant

**Test:**
```rust
#[test]
fn test_meta_type_info_binary_op() {
    let mut interp = ComptimeInterpreter::new();
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer32(2)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Integer32(3)),
    };
    let ast_node = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(expr)));

    let type_info = interp.meta_type_info(ast_node).unwrap();

    // Verify type_info contains expected fields
    // ...
}
```

### Step 3: Implement meta.fields() and meta.variant_name()

**Implementation:**
- Add `meta_fields()` method to `ComptimeInterpreter`
- Add `meta_variant_name()` method
- Register both as comptime intrinsics
- Write tests

**Test:**
```rust
#[test]
fn test_meta_fields() {
    let mut interp = ComptimeInterpreter::new();
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer32(2)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Integer32(3)),
    };
    let ast_node = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(expr)));

    let fields = interp.meta_fields(ast_node).unwrap();

    match fields {
        ComptimeValue::Array(field_list) => {
            assert_eq!(field_list.len(), 3);  // left, op, right
        }
        _ => panic!("Expected array of fields"),
    }
}
```

### Step 4: Write a Minimal Zen Walker

**Create:** `/home/ubuntu/zenlang/stdlib/codegen/walker.zen`

**Implementation:**
```zen
// Minimal walker that just prints AST structure
{ meta, io } = @std

walk_expression = (expr: Expression, depth: u32) void {
    indent = "  ".repeat(depth)
    variant = meta.variant_name(expr)
    io.println(indent + variant)

    // Recurse into children (hardcoded for now)
    expr ?
        | BinaryOp(op) {
            walk_expression(op.left.val, depth + 1)
            walk_expression(op.right.val, depth + 1)
        }
        | FunctionCall(call) {
            call.args.loop((arg) {
                walk_expression(arg, depth + 1)
            })
        }
        | _ { }
}
```

**Test:**
```zen
// Test input
test_walk = () void {
    expr = .BinaryOp({
        left: @std.heap.alloc(.Integer32(2)),
        op: .Add,
        right: @std.heap.alloc(.Integer32(3)),
    })
    walk_expression(expr, 0)
    // Should print:
    // BinaryOp
    //   Integer32
    //   Integer32
}
```

### Step 5: Implement a Minimal C Emitter in Zen

**Create:** `/home/ubuntu/zenlang/stdlib/codegen/c_emitter.zen`

**Implementation:**
```zen
{ meta, string } = @std

CEmitter: {
    buffer:: string.StringBuilder,
}

CEmitter.new = () CEmitter {
    return CEmitter { buffer: string.StringBuilder.new() }
}

emit_expression = (self: CEmitter, expr: Expression) void {
    expr ?
        | Integer32(val) {
            self.buffer.append(val.to_string())
        }
        | BinaryOp(op) {
            emit_expression(self, op.left.val)
            emit_operator(self, op.op)
            emit_expression(self, op.right.val)
        }
        | _ {
            self.buffer.append("/* unhandled */")
        }
}

emit_operator = (self: CEmitter, op: BinaryOperator) void {
    op_str = op ?
        | Add { " + " }
        | Subtract { " - " }
        | Multiply { " * " }
        | Divide { " / " }
        | _ { " ??? " }
    self.buffer.append(op_str)
}

to_c = (expr: Expression) String {
    emitter = CEmitter.new()
    emit_expression(emitter, expr)
    return emitter.buffer.to_string()
}
```

**Test:**
```zen
test_emit_binary_op = () void {
    expr = .BinaryOp({
        left: @std.heap.alloc(.Integer32(2)),
        op: .Add,
        right: @std.heap.alloc(.Integer32(3)),
    })
    c_code = to_c(expr)
    assert(c_code == "2 + 3", "Failed: got " + c_code)
}
```

### Step 6: Expand Coverage (Iterative)

**Expand in this order:**
1. **Expressions:** Add remaining 30 variants
2. **Statements:** Add all 13 variants
3. **Declarations:** Add all 12 variants
4. **Functions:** Add function emission
5. **Structs:** Add struct emission
6. **Enums:** Add enum emission (tagged unions)

**For each category:**
1. Implement `meta.type_info()` support (Rust)
2. Write walker function (Zen)
3. Write emitter methods (Zen)
4. Write tests (Zen and Rust)

### Step 7: End-to-End Test

**Create:** `/home/ubuntu/zenlang/examples/hello_world_c_emit.zen`

**Implementation:**
```zen
// Simple Zen program
main = () void {
    @std.io.println("Hello, world!")
}
```

**Compiler invocation:**
```bash
zen compile --target=c examples/hello_world_c_emit.zen -o hello.c
gcc hello.c -o hello
./hello
# Output: Hello, world!
```

**Verification:**
- Zen parses program → AST
- Zen walks AST with C emitter → C code
- gcc compiles C code → binary
- Binary runs correctly

### Step 8: JavaScript and Python Emitters

**Repeat Steps 5-7 for JS and Python:**
- `/home/ubuntu/zenlang/stdlib/codegen/js_emitter.zen`
- `/home/ubuntu/zenlang/stdlib/codegen/python_emitter.zen`

**Test:**
```bash
zen compile --target=js examples/hello_world_c_emit.zen -o hello.js
node hello.js
# Output: Hello, world!

zen compile --target=python examples/hello_world_c_emit.zen -o hello.py
python3 hello.py
# Output: Hello, world!
```

### Step 9: Optimize and Refine

**Performance optimization:**
- Profile codegen time
- Identify bottlenecks
- Consider JIT compilation of emitters
- Add caching

**API refinement:**
- Simplify walker interface
- Add helper functions for common patterns
- Improve error messages
- Add documentation

### Step 10: Bootstrap Test

**Goal:** Compile the Zen C emitter using itself.

**Steps:**
1. Compile `stdlib/codegen/c_emitter.zen` to C using Rust compiler → `c_emitter.c`
2. Compile `c_emitter.c` with gcc → `c_emitter` (native binary)
3. Use `c_emitter` to compile itself → `c_emitter2.c`
4. Compile `c_emitter2.c` with gcc → `c_emitter2`
5. Verify `c_emitter` and `c_emitter2` produce identical output

**Success criteria:** The Zen compiler can compile its own C backend without relying on Rust.

---

## Conclusion

This architecture transforms code generation from **compiler internals** to **user-level metaprogramming**. By exposing the AST through `meta.type_info()`, Zen enables:

1. **Code generators as Zen programs** - No Rust required
2. **User extensibility** - Anyone can write a backend
3. **Composable transformations** - Optimization passes are functions
4. **Self-hosting path** - Bootstrapping via C output
5. **Multi-target compilation** - Same AST, different emitters

The implementation is incremental, testable, and builds on Zen's existing strengths (pattern matching, behaviors, comptime). The result is a compiler that users can extend, understand, and contribute to - all without touching compiler internals.

**The key insight:** AST nodes are just data structures. If we expose them through `meta`, they become first-class values that Zen code can manipulate. Code generation is just AST traversal + string building. And Zen is **perfect** for that.

