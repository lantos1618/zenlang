# Zen Language TODOs

## ✅ Completed (This Session)
1. ✅ Read lang.md spec to understand requirements
2. ✅ Search and replace all lynlang/lyn references with zen
3. ✅ Verify language implementation matches lang.md spec
4. ✅ Create working .zen examples matching spec
5. ✅ Run tests and verify functionality
6. ✅ Update README with accurate syntax
7. ✅ Consolidate project documentation
8. ✅ Clean up and organize project structure

## Future Development Tasks

### Bug Fixes (High Priority)
- [ ] Fix 6 failing tests in parser_generics
- [ ] Resolve parser_error_recovery test issues

### Core Implementation (Medium Priority)
- [ ] Complete type checker separation from codegen
- [ ] Implement generic type instantiation/monomorphization
- [ ] Finish comptime evaluation engine
- [ ] Implement behaviors/traits system
- [ ] Add @std namespace bootstrap

### Standard Library (Low Priority)
- [ ] Implement core module (@std.core)
- [ ] Add build module (@std.build)
- [ ] Create io module
- [ ] Add collections module
- [ ] Implement memory management module

### Language Features (Future)
- [ ] Async/await support
- [ ] Advanced pattern matching features
- [ ] Compile-time type reflection
- [ ] Package management system
- [ ] Documentation generation

## Principles
- Keep implementation simple and elegant (KISS/DRY)
- Maintain 80/20 rule: 80% features, 20% testing
- Use frequent git commits
- Follow lang.md specification exactly
- Clean up after completion