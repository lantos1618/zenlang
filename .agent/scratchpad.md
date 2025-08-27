# Zen Language - Scratchpad

## Loop Syntax Examples

### Old Style (To Remove)
```zen
loop i in 0..10 { }
loop item in items { }
```

### New Functional Style
```zen
// Range iteration
range(0, 10).loop(|i| {
    printf("i = %d\n", i)
})

// Iterator over collection
items.loop(|item| {
    process(item)
})

// Conditional loop
loop(i < 10, {
    printf("i = %d\n", i)
    i = i + 1
})

// Infinite loop
loop({
    // break when needed
})
```

## Key Zen Patterns

### Variable Declaration
```zen
x := 10        // immutable
y ::= 20       // mutable
```

### Function Declaration
```zen
add := (a: i32, b: i32) -> i32 {
    return a + b
}
```

### Pattern Matching
```zen
value ? {
    Some(x) => printf("Value: %d\n", x),
    None => printf("No value\n")
}
```

### Error Handling
```zen
Result<i32, String>
Option<i32>
```

## Git Commands Reference
```bash
# Check status
git status

# Add changes
git add -A

# Commit with message
git commit -m "feat: Description"

# Push to remote
git push

# Merge to main
git checkout main
git merge feature-branch
```

## Testing Commands
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Notes
- Always test before committing
- Use descriptive commit messages
- Keep functions small and focused
- Document complex logic
- Maintain 100% test pass rate