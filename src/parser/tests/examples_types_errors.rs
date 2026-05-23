use super::*;

fn assert_project_example_parses(path: &str, label: &str) {
    let src = std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {label}: {err}"));
    let prog = parse_str(&src).unwrap_or_else(|errs| {
        for err in &errs {
            eprintln!("{err:?}");
        }
        panic!("{label} parse failed with {} errors", errs.len());
    });

    assert!(
        !prog.declarations.is_empty(),
        "{label} should have declarations, got {}",
        prog.declarations.len()
    );
}

#[test]
fn parse_project_example() {
    assert_project_example_parses("examples/project/main.zen", "project example");
}

#[test]
fn parse_project_build_zen_example() {
    assert_project_example_parses("examples/project/build.zen", "project build.zen");
}

#[test]
fn parse_nested_generics() {
    // Single-level generic
    parse_single_decl("foo = (x: Vec<i32>) void { }");

    // Nested: Vec<Ptr<i32>> — the >> must not be parsed as ShiftRight
    parse_single_decl("bar = (x: Vec<Ptr<i32>>) void { }");

    // Triple-nested: Map<StaticString, Vec<Ptr<f64>>>
    parse_single_decl("baz = (x: Map<StaticString, Vec<Ptr<f64>>>) void { }");

    // Deeply nested: A<B<C<D<i32>>>>
    parse_single_decl("deep = (x: A<B<C<D<i32>>>>) void { }");
}

#[test]
fn parse_slice_type() {
    match parse_single_decl("foo = (s: Slice<i32>) void { }") {
        Declaration::Function { params, .. } => {
            assert!(
                matches!(&params[0].ty, AstType::Slice(inner) if **inner == AstType::I32),
                "expected Slice<I32>, got {:?}",
                params[0].ty
            );
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_string_type_is_named() {
    // String should parse as AstType::Named("String"), NOT AstType::Str
    match parse_single_decl("foo = (s: String) void { }") {
        Declaration::Function { params, .. } => match &params[0].ty {
            AstType::Named(n) => assert_eq!(n, "String"),
            AstType::Str => panic!("String should not parse as Str"),
            other => panic!("expected Named(\"String\"), got {:?}", other),
        },
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_static_string_is_str() {
    // StaticString should still parse as AstType::Str
    match parse_single_decl("foo = (s: StaticString) void { }") {
        Declaration::Function { params, .. } => {
            assert!(
                matches!(&params[0].ty, AstType::Str),
                "expected Str, got {:?}",
                params[0].ty
            );
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

// ── Error / negative parser tests ────────────────────────

#[test]
fn parse_error_missing_closing_brace() {
    let result = parse_str("foo = () void {");
    assert!(
        result.is_err(),
        "expected parse error for missing closing brace"
    );
}

#[test]
fn parse_error_unexpected_token() {
    let result = parse_str("foo = () i32 + +");
    assert!(
        result.is_err(),
        "expected parse error for unexpected tokens"
    );
}
