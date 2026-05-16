use super::*;

mod absence;

#[test]
fn check_program_with_symbols_requires_resolver_parameter_locals() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "a");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver parameter local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'a'")),
        "expected missing resolver parameter local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_parameter_local_mutability() {
    let program = parse_program(
        r#"
add = (mut a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("a", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver parameter local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has mutability immutable, expected mutable")),
        "expected resolver parameter local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_visibility_and_source() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Local, "a", true);
    symbols.set_import_source_for_test(Namespace::Local, "a", Some("std".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local visibility/source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has visibility public, expected private")),
        "expected resolver local visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has source 'std', expected none")),
        "expected resolver local source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_var_decl_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    value = 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver var local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver var local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_var_decl_local_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    value ::= 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("value", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver var local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability immutable, expected mutable")),
        "expected resolver var local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    0
}
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 {
    value = 1
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra local symbol 'value'")),
        "expected extra resolver local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_mutability_by_scope() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let inner_scope = symbols
        .symbols()
        .iter()
        .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
        .map(|symbol| symbol.scope_id)
        .max()
        .expect("inner value local");
    symbols.set_local_mutability_in_scope_for_test("value", inner_scope, Some(true));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver scoped local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability mutable, expected immutable")),
        "expected scoped resolver local mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_pattern_locals() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "inner");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver pattern local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'inner'")),
        "expected missing resolver pattern local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_top_level_expr_locals() {
    let program = parse_program(
        r#"
value := 1
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver top-level expr local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver top-level expr local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_closure_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "inner");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver closure local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'inner'")),
        "expected missing resolver closure local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_closure_parameter_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("input", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver closure parameter mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'input' has mutability immutable, expected mutable")),
        "expected resolver closure parameter mutability diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_struct_field_default_locals() {
    let program = parse_program(
        r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver struct field default local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver struct field default local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_behavior_default_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str {
        value = "{}"
        value
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver behavior default local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver behavior default local diagnostic, got {err:?}"
    );
}
