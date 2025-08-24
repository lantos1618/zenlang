# Enum Variant Handling Improvements

## Current State
- Basic enum parsing works
- Enum variant compilation creates simple struct { tag: i64, payload: i64 }
- TODO comment indicates variant index lookup is missing
- Pattern matching on enums not yet implemented

## Improvements Needed

### 1. Proper Variant Index Management
- Track variant indices in enum definitions
- Look up correct variant index during compilation
- Store enum metadata in symbol table

### 2. Enhanced Payload Support
- Support different payload types (not just i64)
- Handle multiple payload fields
- Support named fields in variants

### 3. Pattern Matching Integration
- Implement enum pattern matching in codegen
- Support exhaustiveness checking
- Handle nested patterns in enum payloads

### 4. Type System Integration
- Proper type checking for enum variants
- Support generic enums (already partially done)
- Validate variant usage

### 5. Memory Layout Optimization
- Tagged union representation
- Optimize for common cases (small enums)
- Support for discriminant values

## Implementation Steps

1. **Fix variant index lookup** (HIGH PRIORITY)
   - Store variant indices in enum definitions
   - Look up indices during compilation
   - Update compile_enum_variant function

2. **Improve payload handling**
   - Support arbitrary types in payloads
   - Handle unit variants properly
   - Support tuple and struct variants

3. **Add pattern matching support**
   - Implement enum pattern compilation
   - Add variant matching logic
   - Support payload extraction

4. **Type checking enhancements**
   - Validate variant names
   - Check payload types
   - Ensure exhaustiveness