use super::*;

mod precollection_tasks;
mod resolver_replay;
mod tasks;

#[test]
fn check_program_rejects_self_type_outside_method_or_behavior() {
    let program = parse_program(
        r#"
main = (value: Self) i32 { 0 }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("Self should require a method or behavior context");

    assert!(
        err.iter()
            .any(|d| d.message.contains("Self type is only valid")),
        "expected invalid Self type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_unknown_type_references() {
    let program = parse_program(
        r#"
main = (value: Missing, items: Bag<i32>) i32 { 0 }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("unknown type reference should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Bag'")),
        "expected unknown generic type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_unknown_type_references_in_struct_field_defaults() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T = {
        same: Missing = 1
        same
    }
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("unknown struct field default type reference should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown field default type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_struct_field_default_type_mismatch() {
    let program = parse_program(
        r#"
Point: { x: i32 = "bad" }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("struct field default type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("field `x` default expects `i32`, found `StaticString`")),
        "expected field default type mismatch diagnostic, got {err:?}"
    );
}
