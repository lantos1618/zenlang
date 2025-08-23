# Lynlang (Zen) Project Plan

## Current Sprint Focus
1. Fix failing tests (5 tests currently failing)
2. Complete parser for advanced features
3. Maintain test coverage at 80% feature work, 20% testing

## Immediate Tasks
### Parser Improvements (Critical)
- [ ] Fix variable declaration parsing edge cases
- [ ] Complete loop condition parsing for complex expressions
- [ ] Implement member access (`.` operator) parsing
- [ ] Add comptime block parsing support
- [ ] Complete pattern matching with `?` operator

### Test Fixes Required
- [ ] Fix infinite loop in test_loop_construct
- [ ] Fix type inference tests
- [ ] Fix function pointer tests
- [ ] Review and fix remaining 3 failing tests

## Development Strategy
- Commit and push after every file edit
- Use agent/ directory for planning and notes
- Email updates to l.leong1618@gmail.com on major milestones
- Focus: 80% feature implementation, 20% testing

## Next Major Milestones
1. Complete parser for all AST features
2. Fix all failing tests
3. Implement type checker
4. Bootstrap standard library (@std namespace)
5. Add generic type support