use super::*;

#[test]
fn test_defer() {
    run_test("defer");
}

#[test]
fn test_defer_early_return() {
    run_test("defer_early_return");
}

#[test]
fn test_boolean_ops() {
    run_test("boolean_ops");
}

#[test]
fn test_nested_structs() {
    run_test("nested_structs");
}

#[test]
fn test_enum_match() {
    run_test("enum_match");
}

#[test]
fn test_mutability() {
    run_test("mutability");
}

#[test]
fn test_recursion() {
    run_test("recursion");
}

#[test]
fn test_nested_match() {
    run_test("nested_match");
}

#[test]
fn test_cast() {
    run_test("cast");
}

#[test]
fn test_multiple_defer() {
    run_test("multiple_defer");
}

#[test]
fn test_stdlib_random() {
    run_test("stdlib_random");
}

#[test]
fn test_stdlib_bits() {
    run_test("stdlib_bits");
}

#[test]
fn test_stdlib_char() {
    run_test("stdlib_char");
}

#[test]
fn test_stdlib_two_namespaces() {
    run_test("stdlib_two_namespaces");
}

#[test]
fn test_stdlib_math() {
    run_test("stdlib_math");
}

#[test]
fn test_stdlib_geometry() {
    run_test("stdlib_geometry");
}

#[test]
fn test_stdlib_ptr() {
    run_test("stdlib_ptr");
}

#[test]
fn test_export_manifest() {
    run_test("export_manifest");
}

#[test]
fn test_ffi_extern() {
    run_test("ffi_extern");
}

#[test]
fn test_stdlib_allocator() {
    run_test("stdlib_allocator");
}

#[test]
fn test_stdlib_vec_allocator() {
    run_test("stdlib_vec_allocator");
}

#[test]
fn test_stdlib_stack_no_alloc_import() {
    run_test("stdlib_stack_no_alloc_import");
}

#[test]
fn test_stdlib_queue_no_alloc_import() {
    run_test("stdlib_queue_no_alloc_import");
}

#[test]
fn test_default_type_param() {
    run_test("default_type_param");
}

#[test]
fn test_stdlib_arena() {
    run_test("stdlib_arena");
}

#[test]
fn test_stdlib_pool() {
    run_test("stdlib_pool");
}

#[test]
fn test_stdlib_heap() {
    run_test("stdlib_heap");
}

#[test]
fn test_stdlib_gpa() {
    run_test("stdlib_gpa");
}

#[test]
fn test_stdlib_prng() {
    run_test("stdlib_prng");
}

#[test]
fn test_stdlib_propagate() {
    run_test("stdlib_propagate");
}

#[test]
fn test_stdlib_getrandom() {
    run_test("stdlib_getrandom");
}

#[test]
fn test_stdlib_slice() {
    run_test("stdlib_slice");
}

#[test]
fn test_stdlib_buffer() {
    run_test("stdlib_buffer");
}

#[test]
fn test_async_await_ready() {
    // An `@async` leaf returning a ready value, and a chained async fn that
    // awaits twice threading a local across the suspends, driven by `block_on`.
    // Proves the milestone-1 state-machine lowering end to end (ASYNC_PLAN.md).
    run_test("async_await_ready");
}
