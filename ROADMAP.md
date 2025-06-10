# Lynlang Compiler Roadmap

## Current Status
✅ Basic compiler infrastructure
✅ LLVM IR generation
✅ Functions, variables, and basic types (int, float, string, void)
✅ Binary operations and comparisons
✅ Conditional expressions (pattern matching)
✅ Function calls
✅ Basic error handling

## Missing Core Features (Priority Order)

### 1. C FFI (Foreign Function Interface) - HIGH PRIORITY
**Why First:** Essential for bootstrapping the standard library and OS interaction
- [ ] External function declarations (`extern`)
- [ ] C-compatible types and calling conventions
- [ ] Linking with C libraries
- [ ] Header file generation for Lynlang functions

### 2. Parser - HIGH PRIORITY
**Why:** Currently using manual AST construction in tests
- [ ] Lexer/Tokenizer
- [ ] Parser (recursive descent or parser combinator)
- [ ] Error recovery and diagnostics
- [ ] Source location tracking

### 3. Additional Type System Features
- [ ] Arrays (`Array<T>`)
- [ ] Structs (record types)
- [ ] Enums (sum types/tagged unions)
- [ ] Pointers (`Ptr<T>`) - unsafe raw pointers
- [ ] References (`Ref<T>`) - managed references
- [ ] Type aliases
- [ ] Option<T> and Result<T,E> types

### 4. Module System
- [ ] Module declarations and imports
- [ ] The `@std` namespace
- [ ] `build.import()` functionality
- [ ] Visibility modifiers (public/private)

### 5. Loop Construct
**From lang.md:** Unified `loop` for all iteration needs
- [ ] Conditional loops (`while`-like)
- [ ] Iterator loops (ranges, collections)
- [ ] Loop control (`break`, `continue`)
- [ ] Labeled loops

### 6. Memory Management
- [ ] Allocator interface/behavior
- [ ] Stack vs heap allocation
- [ ] Custom allocators
- [ ] Reference counting or GC (design decision needed)

### 7. Behaviors (Trait System)
**From lang.md:** Contract-based polymorphism
- [ ] Behavior definitions
- [ ] Implementation blocks
- [ ] Static dispatch (monomorphization)
- [ ] Dynamic dispatch (trait objects)

### 8. Compile-time Metaprogramming (`comptime`)
**From lang.md:** Core feature for zero-cost abstractions
- [ ] `comptime` blocks
- [ ] Compile-time function evaluation
- [ ] Type-level programming
- [ ] Code generation

### 9. String Interpolation
- [ ] `$(expression)` syntax in string literals
- [ ] ToString/Stringer behavior

### 10. Async/Await
- [ ] `async` functions
- [ ] `Task<T>` type
- [ ] `await` expressions
- [ ] Runtime/scheduler integration

### 11. Standard Library
- [ ] Core types and functions
- [ ] I/O operations
- [ ] Collections (List, Map, Set)
- [ ] String utilities
- [ ] Math functions
- [ ] OS interface
- [ ] Memory management utilities

### 12. Advanced Features
- [ ] Closures/lambdas
- [ ] Generics
- [ ] Pattern matching extensions
- [ ] Operator overloading
- [ ] UFCS (Uniform Function Call Syntax)

### 13. Tooling
- [ ] REPL
- [ ] Package manager
- [ ] Build system
- [ ] Debugger support
- [ ] Language server (LSP)
- [ ] Formatter
- [ ] Documentation generator

## Suggested Implementation Order

### Phase 1: Foundation (Next Steps)
1. **C FFI** - Enable system programming and library reuse
2. **Parser** - Move beyond manual AST construction
3. **Basic Structs** - Essential for real programs

### Phase 2: Type System
4. **Arrays and Enums**
5. **Module System**
6. **Pointers and References**

### Phase 3: Control Flow
7. **Loop Construct**
8. **Enhanced Pattern Matching**

### Phase 4: Advanced Features
9. **Behaviors**
10. **Comptime**
11. **Async/Await**

### Phase 5: Polish
12. **Standard Library**
13. **Tooling**

## Next Immediate Steps

Since you mentioned C FFI, that's an excellent choice for the next feature! It would enable us to:
- Call `printf`, `malloc`, etc. for testing
- Build a real standard library
- Interface with existing system libraries
- Make the language immediately useful

Would you like to start implementing C FFI? 