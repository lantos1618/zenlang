# Zen Compiler Development - Prioritized Task List

## Immediate Priority (Self-Hosting Critical Path)

### Phase 1: Language Foundation Completion
**Estimated Timeline: 2-3 months**

#### P0 - Critical Missing Language Features
1. **Complete Struct Codegen** - Currently parsing works but codegen incomplete
   - Struct instantiation: `Person{ name: "Alice", age: 30 }`
   - Field access: `person.name`, `person.age`
   - Mutable field modification: `person.email = Some("alice@example.com")`
   
2. **Complete Pattern Matching Codegen** - Parser done, codegen WIP
   - Implement `?` operator for all patterns
   - Support destructuring: `point ? | { x, y } => use(x, y)`
   - Enum pattern matching: `result ? | .Ok -> value => process(value)`

3. **Generic Instantiation/Monomorphization** - Critical for standard library
   - Complete type instantiation for `Array<T>`, `Option<T>`, `Result<T,E>`
   - Proper monomorphization of generic functions
   - Generic struct/enum support

4. **String Interpolation** - `$(expr)` syntax missing
   - Parser support for embedded expressions
   - Codegen for string formatting
   - Integration with ToString behavior

5. **Module Import System** - Essential for larger programs
   - `build.import("module")` functionality
   - Module resolution and loading
   - Proper namespace isolation

#### P1 - Enhanced Language Features  
6. **Complete Loop Implementation** - Make spec-compliant
   - `loop condition { }` (while-like)
   - `loop item in collection { }` (iteration)
   - `loop i in 0..10 { }` (range-based)
   - Loop control: `break`, `continue`

7. **Complete Enum Implementation** - Sum types essential
   - Enum definition parsing and codegen
   - Variant construction: `.Ok(value)`, `.Err(error)`
   - Memory layout optimization

### Phase 2: Standard Library Development  
**Estimated Timeline: 1-2 months**

#### P0 - Essential Standard Library Components
8. **Memory Management** - Foundation for all data structures
   - Implement `Ptr<T>` and `Ref<T>` types
   - Basic allocator interface
   - Stack vs heap allocation decisions

9. **Collections Library** - Arrays, Lists, Maps
   - `Array<T>` with bounds checking
   - Dynamic `List<T>` (Vec equivalent)
   - `Map<K,V>` hash table implementation
   - Iterator support for all collections

10. **Enhanced IO Module** - Beyond basic console/file operations
    - Formatted output (printf-style)
    - Buffered IO operations
    - Error handling for IO operations
    - Path manipulation utilities

11. **String Utilities** - Essential for compiler work
    - String slicing and manipulation
    - String building/concatenation
    - UTF-8 handling
    - Regular expression support (basic)

#### P1 - System Integration
12. **OS Interface** - Needed for build tools and file operations
    - Process spawning and management
    - Environment variables
    - File system operations
    - Signal handling

13. **Error Handling Utilities** - Improve Result/Option ergonomics
    - Error chaining and context
    - Common error types
    - Debugging utilities (stack traces)

### Phase 3: Advanced Language Features
**Estimated Timeline: 2-3 months**

#### P0 - Compiler Prerequisites
14. **Behaviors (Trait System)** - Polymorphism for compiler abstractions
    - Behavior definitions and implementations
    - Static dispatch (monomorphization)  
    - Dynamic dispatch capability
    - Built-in behaviors (ToString, Clone, etc.)

15. **Complete Comptime System** - Metaprogramming for code generation
    - `comptime` block execution
    - Compile-time function evaluation
    - Type-level programming
    - Code generation capabilities

16. **UFCS Implementation** - Clean API design
    - Method-style syntax: `value.method()`
    - Integration with behaviors
    - Proper precedence and overloading

#### P1 - Advanced Features
17. **Build System Integration** - Project management
    - Package/module management
    - Dependency resolution
    - Build configuration
    - Testing framework integration

18. **LLVM Integration** - Backend flexibility
    - Either complete FFI bindings to LLVM
    - Or custom Zen backend
    - Optimization pass integration
    - Debug information generation

## Self-Hosting Milestone Criteria

### Minimum Viable Self-Hosting Compiler
To write a basic Zen compiler in Zen, we need:
- ✅ Functions, variables, basic types (DONE)
- 🚧 Structs and enums with full codegen
- 🚧 Pattern matching for control flow  
- 🚧 Generic instantiation
- ❌ Module system for code organization
- ❌ Collections (Array, List, Map)
- ❌ String manipulation
- ❌ File IO operations
- ❌ Basic memory management

### Full Feature Parity  
For complete self-hosting with all features:
- ❌ Complete comptime system
- ❌ Behaviors/trait system
- ❌ UFCS
- ❌ Async/await
- ❌ Advanced optimizations

## Development Strategy

### Incremental Approach
1. **Test-First Development**: Each feature needs passing tests before merge
2. **Working Examples**: Create .zen examples that demonstrate each feature
3. **Bootstrap Gradually**: Start with minimal self-hosted compiler, add features incrementally
4. **Maintain Compatibility**: Keep existing working features functional

### Risk Mitigation
- **Parallel Development**: Some features can be developed in parallel branches
- **Fallback Plan**: Current Rust compiler remains functional during transition
- **Feature Flags**: New features behind compile-time flags until stable
- **Incremental Migration**: Port compiler modules one at a time to Zen

## Next Immediate Action
**Start with P0 Item #1**: Complete struct codegen
- High impact (enables data structures)
- Well-defined scope
- Foundation for many other features
- Has working parser implementation to build on
