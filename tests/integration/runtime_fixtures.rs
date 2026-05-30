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
