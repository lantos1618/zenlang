use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_impl_methods() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = SymbolTable::default();
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing impl method resolver symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing value symbol 'Point.stringify'")),
        "expected missing impl method symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_impl_method_signature() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) StaticString { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(
        Namespace::Value,
        "Point.stringify",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver impl method signature mismatch should fail");

    assert!(
        err.iter().any(|d| {
            d.message.contains(
            "resolver value symbol 'Point.stringify' has return type 'i32', expected 'StaticString'"
        )
        }),
        "expected resolver impl method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_impl_function_type_signature() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}

Point: { x: i32 }

Point.implements(Mapper) {
    map = (value: Point, callback: (i32) i32) (i32) i32 {
        callback
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "Point.map", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver impl method function type mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Point.map' has return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver impl method function type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_resolver_impl_method_body_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) StaticString {
        label = "point"
        label
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "label");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver impl method body local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'label'")),
        "expected missing resolver impl method body local diagnostic, got {err:?}"
    );
}
