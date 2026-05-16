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
