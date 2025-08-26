# Zen Language - TODO List

## Immediate Priority (This Session)
- [x] Verify printf/puts test output capture
- [ ] Implement string interpolation $(expr) syntax codegen
- [ ] Fix loop syntax to match specification
- [ ] Complete enum codegen

## Short-term (Next Few Sessions)
- [ ] Implement comptime execution framework
- [ ] Expand standard library:
  - [ ] collections module (Vec, HashMap, etc.)
  - [ ] mem module (allocators)
  - [ ] net module (TCP/UDP)
- [ ] Implement behaviors (trait/interface system)
- [ ] Complete UFCS (Uniform Function Call Syntax)
- [ ] Improve type inference

## Medium-term
- [ ] Memory management (Ptr<T>, Ref<T>, allocators)
- [ ] Async/await with Task<T>
- [ ] Module import system completion
- [ ] Package management system

## Long-term (Self-hosting)
- [ ] Bootstrap compiler in Zen
- [ ] Rewrite standard library in Zen (not Rust)
- [ ] Performance optimizations
- [ ] Documentation generation
- [ ] LSP improvements

## Test Improvements Needed
- [ ] Convert ffi.rs tests to use ExecutionHelper
- [ ] Add more comprehensive pattern matching tests
- [ ] Test error handling paths
- [ ] Benchmark suite

## Known Bugs
- [ ] Some generic instantiations fail with complex types
- [ ] Error messages could be more helpful
- [ ] Memory leaks in certain edge cases

## Documentation
- [ ] Complete language reference
- [ ] Tutorial series
- [ ] Standard library docs
- [ ] Contributor guide