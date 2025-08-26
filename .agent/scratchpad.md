# Technical Scratchpad - Zen Compiler Implementation Notes

## Current Architecture Analysis

### Compiler Pipeline (Based on src/ structure)
```
Source Code (.zen) 
    ↓
Lexer (lexer.rs) → Tokens
    ↓  
Parser (parser/) → AST (ast.rs)
    ↓
Type Checker (typechecker/) → Typed AST
    ↓
Code Generator (codegen/llvm/) → LLVM IR
    ↓
LLVM → Native Binary
```

### Key Implementation Files
- **ast.rs**: Core AST definitions (expressions, statements, types)
- **parser/core.rs**: Main parser logic
- **codegen/llvm/mod.rs**: LLVM code generation coordinator
- **stdlib/mod.rs**: @std namespace implementation

## Critical Technical Challenges for Self-Hosting

### 1. Generic Instantiation Deep Dive
**Current Issue**: Parser handles generic syntax but instantiation incomplete

**What's Missing**:
- Type substitution during instantiation (`T` → `i32`)
- Monomorphization of generic functions (create separate copies)
- Generic constraint checking
- Proper memory layout for generic structs

**Implementation Strategy**:
```zen
// Need to handle this transformation:
Container<T> = { items: []T, capacity: usize }
Container<i32> usage -> generate: Container_i32 = { items: []i32, capacity: usize }
```

### 2. Pattern Matching Codegen Architecture
**Current Issue**: Parser creates pattern AST but codegen incomplete

**Missing Components**:
- Pattern compilation to conditional jumps
- Destructuring assignment code generation  
- Guard clause evaluation
- Exhaustiveness checking

**Technical Approach**:
```rust
// Pseudocode for pattern matching codegen
match pattern {
    Pattern::Variable(name) => bind_variable(name, value),
    Pattern::Literal(lit) => generate_equality_check(lit, value),  
    Pattern::Struct{fields} => {
        for field in fields {
            generate_field_access_and_match(field, value);
        }
    }
}
```

### 3. Module System Design Decisions
**Key Questions**:
- Module loading: Dynamic vs static linking?
- Name mangling strategy for preventing conflicts
- Circular dependency handling
- Compilation unit boundaries

**Proposed Architecture**:
```zen
// Module resolution chain:
comptime {
    build := @std.build
    io := build.import("io")  // Resolves to: std.io or ./io.zen or ~/.zen/io/
}
```

### 4. Memory Management Model
**Options Under Consideration**:
1. **Reference Counting (ARC)**: Simple, predictable, some overhead
2. **Ownership System (Rust-like)**: Zero-cost, complex borrow checker  
3. **Hybrid**: RC for shared data, ownership for unique data
4. **GC**: Simplest for users, runtime overhead

**Current Thinking**: Start with simple RC, add ownership later

### 5. String Interpolation Implementation
**Syntax Target**: `"Hello $(name), you scored $(score)%"`

**Implementation Strategy**:
```rust
// Parse string with embedded expressions
struct StringInterpolation {
    parts: Vec<StringPart>,
}
enum StringPart {
    Literal(String),
    Expression(Expression),
}

// Codegen: Expand to StringBuilder pattern
format_string("Hello ", name, ", you scored ", score, "%")
```

## Self-Hosting Technical Requirements

### Minimum Feature Set for Bootstrap Compiler
1. **File IO**: Read .zen source files
2. **String Processing**: Tokenization and parsing
3. **Data Structures**: AST representation (structs, enums)
4. **Collections**: Arrays/Lists for token streams, symbol tables
5. **Hash Maps**: Symbol resolution, type environments
6. **Error Handling**: Result types for parse errors
7. **System Interface**: File paths, command line arguments

### Advanced Features for Full Compiler
1. **LLVM Integration**: FFI bindings or custom backend
2. **Parallel Compilation**: Multi-threaded builds
3. **Incremental Compilation**: Only rebuild changed modules
4. **Debug Information**: Source maps, stack traces
5. **Optimization**: Dead code elimination, inlining

## Performance Considerations

### Compilation Speed Targets
- **Small Programs** (<1000 lines): <1 second
- **Medium Programs** (1000-10000 lines): <10 seconds  
- **Large Programs** (10000+ lines): <60 seconds
- **Self-Compilation**: <5 minutes

### Memory Usage
- **Parser**: Should handle 100MB+ source files
- **Type Checker**: Symbol tables for large programs
- **Code Generator**: Streaming codegen to avoid memory spikes

## Incremental Development Strategy

### Phase 1 Implementation Order
1. **Struct Field Access** (foundation for all data structures)
2. **Basic Pattern Matching** (enables control flow)
3. **Generic Instantiation** (enables standard library)
4. **String Interpolation** (quality of life improvement)

### Testing Strategy  
```bash
# For each feature:
1. Create failing test case
2. Implement minimal feature to pass test
3. Add comprehensive test coverage
4. Create working example in examples/
5. Update documentation
```

## Code Quality Principles

### Error Handling Philosophy
- No panics in compiler code
- All errors return Result<T, CompileError>
- Rich error messages with source locations
- Recoverable parsing errors when possible

### Performance Guidelines
- Prefer stack allocation over heap when possible
- Use string interning for identifiers
- Lazy evaluation for expensive computations  
- Profile-guided optimization for hot paths

## Open Technical Questions

### 1. LLVM Integration Strategy
**Options**:
- A) FFI bindings to LLVM C API
- B) Custom backend (more work, more control)
- C) Cranelift instead of LLVM

**Decision Factors**: Complexity, performance, maintainability

### 2. Build System Architecture
**Requirements**:
- Module dependency resolution
- Incremental compilation
- Package management integration
- Cross-compilation support

### 3. Standard Library Organization
**Questions**:
- How many modules in @std?
- What belongs in core vs extended library?
- Binary size implications of large stdlib

## Implementation Milestones

### Milestone 1: Data Structures (4 weeks)
- [ ] Complete struct codegen
- [ ] Basic enum support  
- [ ] Pattern matching for simple cases
- [ ] Working struct examples

### Milestone 2: Generics (6 weeks)  
- [ ] Generic instantiation working
- [ ] Option<T> and Result<T,E> fully functional
- [ ] Generic collections (Array<T>)
- [ ] Generic function monomorphization

### Milestone 3: Standard Library (8 weeks)
- [ ] String interpolation
- [ ] Basic collections (List, Map)
- [ ] File IO operations
- [ ] Memory management primitives

This scratchpad captures the technical implementation details needed to guide development toward self-hosting capability.
