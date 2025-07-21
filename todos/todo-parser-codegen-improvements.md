# Parser & Codegen Improvements Todo

## 🚨 Critical Parser Gaps

The parser is currently a major bottleneck. Many advanced language features are either unimplemented or only partially parsed.

### Struct & Enum Definitions
- [x] **Complete struct definition parsing**
  - [x] Parse struct fields with types and visibility modifiers
  - [ ] Handle struct methods and associated functions
  - [ ] Support generic struct definitions
  - [ ] Parse struct inheritance/composition

- [x] **Complete enum definition parsing**
  - [x] Parse enum variants with payload types
  - [ ] Handle enum methods and associated functions
  - [ ] Support generic enum definitions
  - [ ] Parse enum discriminants

### Pattern Matching & Conditionals
- [ ] **Implement full pattern matching syntax**
  - [ ] Parse `?` conditional operator
  - [ ] Handle `->` bindings in pattern arms
  - [ ] Support optional guards within pattern matching
  - [ ] Parse destructuring patterns (struct, tuple, array)
  - [ ] Handle literal patterns and identifier patterns

- [x] **Complete loop syntax parsing**
  - [x] Handle loop labels and break/continue
  - [ ] Support range-based loops
  - [x] Parse iterator-based loops

### Advanced Language Features
- [ ] **Implement comptime block parsing**
  - [ ] Parse `comptime` expressions and blocks
  - [ ] Handle compile-time function calls
  - [ ] Support compile-time type operations
  - [ ] Parse conditional compilation directives

- [ ] **Complete expression parsing**
  - [ ] Handle all operator precedence correctly
  - [ ] Parse method calls and field access
  - [ ] Support array indexing and slicing
  - [ ] Handle type annotations in expressions

## 🔧 Codegen Simplifications

While placeholders exist for complex AST nodes, their LLVM IR generation is often simplified, limiting expressiveness.

### Struct & Enum Codegen
- [ ] **Implement proper struct codegen**
  - [ ] Generate correct field offsets and alignment
  - [ ] Handle struct methods and vtables
  - [ ] Support generic struct instantiation
  - [ ] Implement struct copying and moving

- [ ] **Implement proper enum codegen**
  - [ ] Use LLVM unions or tagged structs for payloads
  - [ ] Generate proper discriminant handling
  - [ ] Support enum methods and associated functions
  - [ ] Handle enum pattern matching in codegen

### Pattern Matching Codegen
- [ ] **Develop robust pattern matching codegen**
  - [ ] Generate code for `?` conditional operator
  - [ ] Handle different pattern types (literals, identifiers, destructuring)
  - [ ] Implement pattern guards
  - [ ] Optimize pattern matching with jump tables where possible

### Comptime Evaluation
- [ ] **Implement compile-time execution engine**
  - [ ] Evaluate comptime expressions at compile time
  - [ ] Generate constants from comptime evaluation
  - [ ] Support compile-time type operations
  - [ ] Handle conditional compilation

### Advanced Type Codegen
- [ ] **Improve complex type handling**
  - [ ] Generate proper LLVM types for generics
  - [ ] Handle trait objects and dynamic dispatch
  - [ ] Implement proper pointer and reference types
  - [ ] Support array and slice types correctly

## 🎯 Type System Implementation

The rich AstType definitions are not fully reflected in LLVM type mapping or type checking logic.

### Type Checker Integration
- [ ] **Introduce dedicated type checking pass**
  - [ ] Create separate type checker after parsing, before codegen
  - [ ] Implement precise type error messages with source spans
  - [ ] Add semantic correctness validation
  - [ ] Handle type inference for complex expressions

### Type Mapping Improvements
- [ ] **Enhance LLVM type mapping**
  - [ ] Map all AstType variants to proper LLVM types
  - [ ] Handle generic type instantiation
  - [ ] Support trait bounds and constraints
  - [ ] Implement proper type coercion rules

### Type Safety
- [ ] **Strengthen type safety**
  - [ ] Validate function call argument types
  - [ ] Check assignment compatibility
  - [ ] Verify pattern matching exhaustiveness
  - [ ] Handle type aliases and newtypes

## 🧪 Test Suite Completeness

Many parser tests are commented out or expect errors, and some codegen tests only verify compilation, not execution.

### Parser Tests
- [ ] **Enable and fix parser tests**
  - [ ] Uncomment failing tests in `tests/parser.rs`
  - [ ] Add tests for struct and enum parsing
  - [ ] Test pattern matching syntax
  - [ ] Verify comptime block parsing
  - [ ] Test loop syntax parsing

### Codegen Tests
- [ ] **Expand codegen test coverage**
  - [ ] Add execution tests, not just compilation
  - [ ] Test struct and enum codegen
  - [ ] Verify pattern matching codegen
  - [ ] Test comptime evaluation
  - [ ] Add integration tests for complex features

### Type System Tests
- [ ] **Add type checking tests**
  - [ ] Test type inference
  - [ ] Verify type error messages
  - [ ] Test generic type instantiation
  - [ ] Validate trait bounds

## 🖥️ REPL Limitations

The current REPL compiles to IR but doesn't execute the code, limiting its interactive utility.

### REPL Execution
- [ ] **Integrate JIT execution engine**
  - [ ] Add LLVM JIT compilation to REPL
  - [ ] Execute compiled code immediately
  - [ ] Support function calls and expressions
  - [ ] Handle variable declarations and assignments

### REPL Features
- [ ] **Enhance REPL functionality**
  - [ ] Add command history and line editing
  - [ ] Support multi-line input
  - [ ] Add help system and documentation
  - [ ] Implement tab completion

### REPL Debugging
- [ ] **Add debugging capabilities**
  - [ ] Show generated IR on demand
  - [ ] Display type information
  - [ ] Add step-by-step execution
  - [ ] Support breakpoints and inspection

## 📋 Implementation Priority

### Phase 1: Core Parser (High Priority)
1. Complete struct and enum definition parsing
2. Implement pattern matching syntax
3. Add loop syntax parsing
4. Enable and fix parser tests

### Phase 2: Type System (High Priority)
1. Introduce dedicated type checker
2. Improve LLVM type mapping
3. Add type safety validation
4. Implement type inference

### Phase 3: Codegen (Medium Priority)
1. Implement proper struct and enum codegen
2. Develop pattern matching codegen
3. Add comptime evaluation
4. Expand codegen tests

### Phase 4: REPL (Medium Priority)
1. Integrate JIT execution
2. Add REPL features
3. Implement debugging capabilities

### Phase 5: Advanced Features (Low Priority)
1. Optimize pattern matching
2. Add advanced comptime features
3. Implement complex type system features

## 🔍 Current Status

- **Parser**: ~65% complete - struct/enum parsing ✅, loop syntax ✅, basic expressions ✅, pattern matching ❌, comptime ❌
- **Codegen**: ~30% complete - basic types work, complex features simplified
- **Type System**: ~20% complete - basic type checking, advanced features missing
- **Tests**: ~65% complete - 15/23 parser tests passing, struct/enum/loop tests ✅, variable declaration issues ❌
- **REPL**: ~15% complete - compiles to IR only, no execution

### Recent Achievements ✅
- **Struct parsing**: Complete implementation with field parsing
- **Enum parsing**: Complete implementation with variant parsing  
- **Loop syntax**: Full support for `loop condition` and `loop x in collection`
- **Break/Continue**: Label support implemented
- **Program parsing**: Proper detection of struct/enum vs function declarations

## 🚨 Next Priorities (Based on Failing Tests)

### Critical Issues to Fix
1. **Variable Declaration Parsing** ❌
   - `test_parse_variable_declaration` - type inference vs explicit types
   - `test_parse_all_variable_declaration_syntax` - comment parsing issue
   - `test_parse_variable_declaration_syntax_separately` - colon parsing

2. **Loop Condition Parsing** ❌
   - `test_parse_loop_with_condition` - condition parsing incomplete
   - `test_parse_loop_with_return` - same issue

3. **Member Access Parsing** ❌
   - `test_parse_member_access` - dot operator not handled

4. **Comptime Block Parsing** ❌
   - `test_parse_comptime_block` - comptime keyword not recognized

5. **Function Return Type Issues** ❌
   - `test_parse_function_with_return` - type inference differences

### Immediate Action Items
1. Fix variable declaration parsing to handle all syntax variants
2. Complete loop condition parsing for complex expressions
3. Add member access (dot operator) parsing
4. Implement comptime block parsing
5. Fix type inference consistency issues

## 📝 Notes

- Focus on parser development first as it's the bottleneck
- Type checker should be introduced early to catch errors before codegen
- Test-driven development approach recommended
- Consider incremental implementation to maintain working state 